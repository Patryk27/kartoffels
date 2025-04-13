use crate::{AliveBot, BotMmioContext, MessageBuffer};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

/// Radio module for the kartoffel bots
///
/// Like other modules the functionality is defined by 3 important methods:
/// - mmio_load
/// - mmio_store
/// - tick
///
/// The module has two unique memory locations for reading and writing:
/// - The incoming message buffer
/// - The outgoing message buffer
///
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct BotRadio {
    /// Can only send when cooldown is 0
    /// scans create different cooldowns per range
    /// this is also equivilent to the *power* they are sent out with at the moment
    /// | Range | Ticks | +-% |
    /// | --- | --- | --- |
    /// | 3 | 20_000 | 10 |
    /// | 5 | 23_000 | 15 |
    /// | 7 | 28_000 | 25 |
    /// | 9 | 35_000 | 30|
    cooldown: u32,
    /// This is a receiving message buffer, it can store 512 bytes of messages each one being a minimum of 4 bytes
    /// It's a circular buffer and has a slightly complicated read functionality
    /// The functionality for reading this is documented under [`mmio_load`][`BotBluetooth::mmio_load`]
    ///
    /// <div class="warning">A big warning!</div>
    ///
    /// To support reading and writing across threads it has been placed in a RwLock to make sure this program never deadlocks
    /// you should *NEVER* read from the MessageBuffer when writing to another one at the same time
    messages: Arc<RwLock<MessageBuffer>>,
    /// This is the buffer holding the current message you are going to send at the moment this is written to at the memory addresses 1..128
    /// This is the message filter, any message recieved is first checked against this (if it is Some), any message not matching this filter is discarded
    /// Since messages can have random decay there is a chance this part of the message mangles and valid messages are dropped as they are seen as invalid (although to reduce this earlier bits in a message are less likely to be mangled)
    pub filter: Option<[u8; 4]>,
}

impl BotRadio {
    /// This function runs each tick!
    pub fn tick(&mut self) {
        self.cooldown = self.cooldown.saturating_sub(1);
    }

    /// This is the endpoint for reading from the bluetooth module
    /// Every increase in 1 by the rdi offset increases the the addr by 4
    /// | Addr beyond 7168 | What it returns |
    /// | --- | --- |
    /// | 0 | radio status (break down bellow) |
    /// | 1..128 | read the current message ready to send |
    /// | 128..256| read from the front of the message buffer |
    ///
    /// Break down of radio status:
    /// In the first byte:
    /// - First bit: is the radio on?
    /// - Second bit: is the radio ready to send?
    /// - Third bit: are there messages ready to read?
    ///
    /// In the third byte:
    /// - How long is the front message?
    ///
    pub fn mmio_load(&self, addr: u32) -> Result<u32, ()> {
        match addr {
            AliveBot::MEM_RADIO => Ok(self.radio_status()),
            addr if addr >= AliveBot::MEM_RADIO + 4 => {
                // Reading from the actual radio module data
                let idx = (addr - AliveBot::MEM_RADIO - 4) as usize;
                let messages = self.messages.read().unwrap();
                messages.mmio_read(idx).map(|v| u32::from_le_bytes(v))
            }
            _ => Err(()),
        }
    }

    fn radio_status(&self) -> u32 {
        // first bit is for if it's on which atm is always 1
        let messages = self.messages.read().unwrap();
        let mut out = 1u16;
        if self.cooldown == 0 {
            out += 2;
        }
        if !messages.is_empty() {
            out += 4;
        }
        (messages.front_message_length() as u32) << 16 | out as u32
    }

    pub fn receive_message(&self, message: &[u8]) -> Result<(), ()> {
        if let Some(filter) = self.filter {
            // basic filtering
            for i in 0..4 {
                if message[i] != filter[i] {
                    return Ok(());
                }
            }
        }
        let mut messages = self.messages.write().unwrap();
        messages
            .write_to_recieve(message)
            .map(|_| ())
            .map_err(|_| ())
    }

    /// This is for writing to the bluetooth module currently we don't care about the addr (beyond the BLUETOOTH offset itself)
    /// The first byte dictates what the write will do
    /// | First byte | What it does |
    /// | --- | --- |
    /// | 0x01 | radio on / off (not yet implemented) |
    /// | 0x02 | send current item in send buffer |
    /// | 0x03 | either half of a message filter (controlled by the second byte either being 0x00 or 0x01) or 0x02 remove the filter
    /// Writing the next 128 bytes is the send buffer
    pub fn mmio_store(
        &mut self,
        ctxt: &mut BotMmioContext,
        addr: u32,
        val: u32,
    ) -> Result<(), ()> {
        match (addr, val.to_le_bytes()) {
            (AliveBot::MEM_RADIO, [0x01, _, _, _]) => Ok(()), // this is for turning the radio on / off
            (AliveBot::MEM_RADIO, [0x02, message_len, power, _]) => {
                if self.cooldown == 0 {
                    let range = BotBluetoothRange::new(power).ok_or(())?;
                    self.send_message(ctxt, range, message_len as usize);
                }
                Ok(())
            }
            (AliveBot::MEM_RADIO, [0x03, mode, a, b]) => {
                self.set_filter(mode, a, b);
                Ok(())
            }
            (AliveBot::MEM_RADIO, [0x04, index, _, _]) => {
                let mut messages = self.messages.write().unwrap();
                messages.remove_recieve(index as usize)
            }
            (idx, data) if idx == AliveBot::MEM_RADIO + 1 => {
                let mut messages = self.messages.write().unwrap();
                messages.mmio_write(
                    (addr - (AliveBot::MEM_RADIO + 1)) as usize,
                    &data,
                )
            }
            _ => Err(()),
        }
    }

    fn set_filter(&mut self, mode: u8, a: u8, b: u8) {
        self.filter = match mode {
            0 => Some(self.filter.map_or_else(
                || [a, b, 0, 0],
                |mut f| {
                    f[0] = a;
                    f[1] = b;
                    f
                },
            )),
            1 => Some(self.filter.map_or_else(
                || [0, 0, a, b],
                |mut f| {
                    f[2] = a;
                    f[3] = b;
                    f
                },
            )),
            2 => None,
            _ => self.filter,
        }
    }

    fn send_message(
        &mut self,
        ctxt: &mut BotMmioContext,
        range: BotBluetoothRange,
        message_length: usize,
    ) {
        if message_length > 128 {
            return;
        }
        let messages = self.messages.read().unwrap();
        let message = messages.send_message[0..message_length].to_vec();
        ctxt.msgs.add_message(message, ctxt.pos, range.len());
        self.cooldown = range.cooldown(ctxt);
    }
}

impl Default for BotRadio {
    fn default() -> Self {
        Self {
            cooldown: 0,
            messages: Arc::new(RwLock::new(MessageBuffer::new())),
            filter: None,
        }
    }
}

/// This is a copy of the radar range code, maybe consider breaking it out
/// This will also be changed a bit for the new method of message power etc.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum BotBluetoothRange {
    D3 = 3,
    D5 = 5,
    D7 = 7,
    D9 = 9,
}

impl BotBluetoothRange {
    fn new(r: u8) -> Option<Self> {
        match r {
            3 => Some(Self::D3),
            5 => Some(Self::D5),
            7 => Some(Self::D7),
            9 => Some(Self::D9),
            _ => None,
        }
    }
    fn len(&self) -> i32 {
        *self as i32
    }
    fn cooldown(&self, ctxt: &mut BotMmioContext) -> u32 {
        match self {
            BotBluetoothRange::D3 => ctxt.cooldown(20_000, 10),
            BotBluetoothRange::D5 => ctxt.cooldown(23_000, 15),
            BotBluetoothRange::D7 => ctxt.cooldown(28_000, 25),
            BotBluetoothRange::D9 => ctxt.cooldown(35_000, 30),
        }
    }
}

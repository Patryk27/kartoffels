use crate::{AliveBot, BotMmioContext};
use glam::{ivec2, IVec2};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

// This is a simple implementation for the bluetooth concept
// Currently the messages are a fixed size and the cooldown is based on distance sent
//
// I would like to switch some functionality around,
// The write buffer should be interacted more like an actual memory location, you should be able to read from it in mmio_load
// and writing should probably work by using a byte offset and writing to the buffer with the [u8;4]
//
// Also I think it's worth (throughout both bluetooth files) makinng sure the read buffer and write buffer are better distinguished
//

/// Bluetooth module for the kartoffel bots
///
/// Like other modules the functionality is defined by 3 important methods:
/// - mmio_load
/// - mmio_store
/// - tick
///
/// The module has two main memory locations:
/// - The incoming message buffer
/// - The outgoing message buffer
///
/// When sending a message there are 4 settings equivalent to the radar scans any bots in that area will (if their incoming message buffer isn't full) receive that message
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct BotBluetooth {
    /// Can only send when cooldown is 0
    /// scans create different cooldowns per range
    /// | Range | Ticks | +-% |
    /// | --- | --- | --- |
    /// | 3 | 20_000 | 10 |
    /// | 5 | 23_000 | 15 |
    /// | 7 | 28_000 | 25 |
    /// | 9 | 35_000 | 30|
    cooldown: u32,
    /// This is a receiving message buffer, it can store 5 messages.
    /// It's a circular buffer and has a slightly complicated read functionality
    /// The functionality for reading this is documented under [`mmio_load`][`BotBluetooth::mmio_load`]
    ///
    /// <div class="warning">A big warning!</div>
    ///
    /// To support reading and writing across threads it has been placed in a RwLock to make sure this program never deadlocks
    /// you should *NEVER* read from the MessageBuffer when writing to another one at the same time
    messages: Arc<RwLock<MessageBuffer>>,
    /// This is the buffer holding the current message you are going to send at the moment this is written two by writing [0x02,index,value,_] to offset 0 where 0 <= index < 32 and value is any u8
    out_message: Message,
}

/// This is the current implementation of a message, obviously this restricts bots to only messaging 32 bytes at a time
/// An interesting alternative could be to change the cooldown based on message size, this would mean we would need to change read and write functionality quite drastically as well
#[derive(Debug, Clone, Deserialize, Serialize, Copy, Default)]
pub struct Message {
    pub sender_id: u64,
    pub message: [u8; 32],
}

impl Message {
    /// This is used to reduce the code's reliance on the message being 40 bytes long
    /// In theory if you wanted to change a message to be 64 bytes for instance then changing these constants would do most of the work (you should make sure you change the bot's API bluetooth code as well as it is more hard coded on that end )
    pub const BYTES: usize = 40;

    /// When given a address return 4 bytes, the rdi function that drives this can only read every 4 bytes meaning we should always receive a multiple of 4
    /// The first 32 bytes returned are the actual message the remaining 8 make up the u64 bot id
    pub fn read(&self, addr: usize) -> Result<[u8; 4], ()> {
        match addr {
            0..=28 => Ok(self.message[addr..addr + 4].try_into().unwrap()),
            32 => {
                let id_front: u32 = (self.sender_id >> 32) as u32;
                Ok(id_front.to_le_bytes())
            }
            36 => Ok((self.sender_id as u32).to_le_bytes()),
            _ => Err(()),
        }
    }

    pub fn write(&mut self, addr: usize, val: u8) -> Result<(), ()> {
        if addr >= self.message.len() {
            return Err(());
        }
        self.message[addr] = val;
        Ok(())
    }

    pub fn clear(&mut self) {
        self.message = [0; 32];
    }
}

/// This is a custom circular buffer written for holding the messages
/// Consider making this Generic and generally just *better* and probably breaking out into it's own file maybe moving read functionality around a bit as well
#[derive(Debug, Clone, Deserialize, Serialize, Copy)]
pub struct MessageBuffer {
    pub buffer: [Message; 5],
    pub front: usize,
    pub length: usize,
}

impl MessageBuffer {
    pub fn new() -> Self {
        MessageBuffer {
            buffer: [Message::default(); 5],
            front: 0,
            length: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub fn is_full(&self) -> bool {
        self.length == self.buffer.len()
    }

    pub fn write(&mut self, v: Message) -> Result<(), ()> {
        if self.is_full() {
            return Err(());
        }
        let address = (self.front + self.length) % self.buffer.len();
        self.buffer[address] = v;
        self.length += 1;
        Ok(())
    }

    pub fn pop(&mut self) -> Result<(), ()> {
        // Consider renaming as it isn't really a pop (it doesn't return anything)
        if self.is_empty() {
            return Err(());
        }
        self.front = (self.front + 1) % self.buffer.len();
        self.length -= 1;
        Ok(())
    }
}

impl BotBluetooth {
    /// This function runs each tick!
    pub fn tick(&mut self) {
        self.cooldown = self.cooldown.saturating_sub(1);
    }

    /// This is the endpoint for reading from the bluetooth module
    /// Every increase in 1 by the rdi offset increases the the addr by 4
    /// | Addr beyond 7168 | What it returns |
    /// | --- | --- |
    /// | 0 | is ready to send a message? |
    /// | 1 | info on the received message buffer [index of the front of the buffer, length of the buffer (how many messages you have), is received message buffer empty related to the last value, number of bytes of each message] |
    /// | 2..| reading straight from the message buffer memory, make sure to add front * 40 to this value to get the oldest message in the buffer|
    ///
    /// Currently you can't read from the write buffer
    pub fn mmio_load(&self, addr: u32) -> Result<u32, ()> {
        match addr {
            AliveBot::MEM_BLUETOOTH => Ok((self.cooldown == 0) as u32),
            addr if addr >= AliveBot::MEM_BLUETOOTH + 4 => {
                // they want to read from the message buffer
                let idx = addr - (AliveBot::MEM_BLUETOOTH + 4);
                let byte_group = self.read(idx as usize)?;
                let out = u32::from_le_bytes(byte_group);
                Ok(out)
            }
            _ => Err(()),
        }
    }

    // the first byte is the received message buffer info [front_address,length,is_empty,how many bytes per message]
    // beyond that is message 1, 2, 3, 4, and 5
    fn read(&self, addr: usize) -> Result<[u8; 4], ()> {
        let messages = self.messages.read().unwrap();
        if addr == 0 {
            let mut out: [u8; 4] = [0; 4];
            out[0] = messages.front as u8;
            out[1] = messages.length as u8;
            out[2] = messages.is_empty() as u8;
            out[3] = Message::BYTES as u8;
            return Ok(out);
        }
        if messages.is_empty() {
            return Err(());
        }
        let fixed_addr = addr - 4; // We now want to start indexing the message buffer's messages
        let message_number: usize = fixed_addr / Message::BYTES;
        let inner_addr = fixed_addr % Message::BYTES;
        messages.buffer[message_number].read(inner_addr)
    }

    fn receive_message(&self, message: &Message) -> Result<(), ()> {
        let mut messages = self.messages.write().unwrap();
        messages.write(*message)
    }

    /// Cleans up the receive buffer (move the pointers forward)
    fn remove_front(&self) -> Result<(), ()> {
        let mut messages = self.messages.write().unwrap();
        if messages.is_empty() {
            return Err(());
        }
        let _ = messages.pop();
        Ok(())
    }

    /// This is for writing to the bluetooth module currently we don't care about the addr (beyond the BLUETOOTH offset itself)
    /// The first byte dictates what the write will do
    /// | First byte | What it does |
    /// | --- | --- |
    /// | 0x01 | send a bluetooth message in a range (dictated by 2nd byte |
    /// | 0x02 | write a byte to the send buffer at at the index  [0x02 , index, byte, _ ] |
    /// | 0x03 | clear the send buffer |
    /// | 0x04 | move the circular buffer forward 1 (pop but without a return)
    pub fn mmio_store(
        &mut self,
        ctxt: &mut BotMmioContext,
        addr: u32,
        val: u32,
    ) -> Result<(), ()> {
        match (addr, val.to_le_bytes()) {
            (AliveBot::MEM_BLUETOOTH, [0x01, range, _, _])
                if let Some(range) = BotBluetoothRange::new(range) =>
            {
                // the bot has been told to send a bluetooth message
                if self.cooldown == 0 {
                    self.send_message(ctxt, range)
                }
                Ok(())
            }
            (AliveBot::MEM_BLUETOOTH, [0x02, index, v, _]) => {
                // we write to the out_message
                self.out_message.write(index as usize, v)
            }
            (AliveBot::MEM_BLUETOOTH, [0x03, _, _, _]) => {
                self.out_message.clear();
                Ok(())
            }
            (AliveBot::MEM_BLUETOOTH, [0x04, _, _, _]) => self.remove_front(),
            _ => Err(()),
        }
    }

    /// It's worth noting that because the [mmio::tick] function that calls this uses `bots.alive.take(idx)` the current bot is replaced in [`Bots.alive`] with a None,
    /// this means you can't access yourself through the bots list which .get uses, the best way to avoid doing this is just to avoid checking the coordinate you are currently at
    /// as it will return a bot id, think there is a bot there, but midway through the get function unwrap a None
    /// This doesn't matter for us since we want to avoid messaging ourself anyway but it's worth noting anyway as it took a while to find out what was breaking for me
    fn send_message(
        &mut self,
        ctxt: &mut BotMmioContext,
        range: BotBluetoothRange,
    ) {
        let self_id = ctxt.bots.lookup_at(ctxt.pos).unwrap().get().get();
        self.out_message.sender_id = self_id;

        for y in 0..range.len() {
            for x in 0..range.len() {
                let pos = {
                    let offset = ivec2(x as i32, y as i32)
                        - IVec2::splat(range.len() as i32) / 2;
                    ctxt.pos + ctxt.dir.as_vec().rotate(offset.perp())
                };

                if let Some(bot_id) = ctxt.bots.lookup_at(pos) {
                    let bot = ctxt.bots.get(bot_id).unwrap();
                    let _ = bot.bluetooth.receive_message(&self.out_message);
                }
            }
        }
        self.cooldown = range.cooldown(ctxt);
    }
}

impl Default for BotBluetooth {
    fn default() -> Self {
        Self {
            cooldown: 0,
            messages: Arc::new(RwLock::new(MessageBuffer::new())),
            out_message: Message::default(),
        }
    }
}

/// This is a copy of the radar range code, maybe consider breaking it out
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
    fn len(&self) -> u32 {
        *self as u32
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
// TODO TESTS

use crate::{AliveBot, BotMmioContext};
use glam::{ivec2, IVec2};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use tracing::{self, info, trace};

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct BotBluetooth {
    cooldown: u32,
    messages: Arc<RwLock<MessageBuffer>>, // I've stuck an Arc and RwLock on this buffer atm this seems like the way to write it
    // but I should really test it as well I think it will work though
    out_message: Message,
}

#[derive(Debug, Clone, Deserialize, Serialize, Copy, Default)]
pub struct Message {
    pub sender_id: u64,
    pub message: [u8; 32],
}

impl Message {
    pub fn len() -> usize {
        32 + 8
    }

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
        if self.is_empty() {
            return Err(());
        }
        self.front = (self.front + 1) % 5;
        self.length -= 1;
        Ok(())
    }
}

impl BotBluetooth {
    pub fn tick(&mut self) {
        self.cooldown = self.cooldown.saturating_sub(1);
    }

    // Every increase in 1 by the rdi offset increases the the addr by 4
    pub fn mmio_load(&self, addr: u32) -> Result<u32, ()> {
        match addr {
            AliveBot::MEM_BLUETOOTH => Ok((self.cooldown == 0) as u32),
            addr if addr >= AliveBot::MEM_BLUETOOTH + 4 => {
                let idx = addr - (AliveBot::MEM_BLUETOOTH + 4);
                let byte_group = self.read(idx as usize)?;
                Ok(u32::from_le_bytes(byte_group))
            }
            _ => Err(()),
        }
    }

    // the first byte is the recieved message buffer info [front_address,length,is_empty,how many bytes per mesasge]
    // beyond that is message 1, 2, 3, 4, and 5
    fn read(&self, addr: usize) -> Result<[u8; 4], ()> {
        let messages = self.messages.read().unwrap();
        if addr == 0 {
            let mut out: [u8; 4] = [0; 4];
            out[0] = messages.front as u8;
            out[1] = messages.length as u8;
            out[2] = messages.is_empty() as u8;
            out[3] = Message::len() as u8;
            return Ok(out);
        }
        if messages.is_empty() {
            return Err(());
        }
        let message_number: usize = addr / Message::len();
        let inner_addr = addr % Message::len();
        messages.buffer[message_number].read(inner_addr)
    }

    fn recieve_message(&self, message: &Message) -> Result<(), ()> {
        let mut messages = self.messages.write().unwrap(); // deal with .unwrap later
        messages.write(*message)
    }

    fn remove_front(&self) -> Result<(), ()> {
        let mut messages = self.messages.write().unwrap();
        if messages.is_empty() {
            return Err(());
        }
        let _ = messages.pop();
        Ok(())
    }

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
            // TODO: Maybe add a feature for checking if a bot has an open spot in their buffer
            _ => Err(()),
        }
    }

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
                    let bot = ctxt.bots.get(bot_id).unwrap(); // the bot might die inbetween these instructions??
                    let _ = bot.bluetooth.recieve_message(&self.out_message); // deal with this as well
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

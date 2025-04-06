use crate::{AliveBot, BotMmioContext};
use serde::{Deserialize, Serialize};
use std::array::TryFromSliceError;
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
    out_message: Vec<u8>,
    /// This is the message filter, any message recieved is first checked against this (if it is Some), any message not matching this filter is discarded
    /// Since messages can have random decay there is a chance this part of the message mangles and valid messages are dropped as they are seen as invalid (although to reduce this earlier bits in a message are less likely to be mangled)
    pub filter: Option<[u8; 4]>,
}

/// For dealing with message pointers in the [`MessageBuffer`]
/// just hides some logic away in a nice neat section
#[derive(Debug, Clone, Deserialize, Serialize, Copy)]
pub struct MessagePointer {
    pub ptr: usize,
    pub length: usize,
}

impl MessagePointer {
    pub fn as_mmio_output(&self) -> [u8; 4] {
        let ptr_bytes = (self.ptr as u16).to_le_bytes(); // Since there are 512Bytes in the buffer the address for every message can't be stored in a u8, we instead have to cast it down to a u16 annoyingly we only need one bit of these extra 8 we use
        [ptr_bytes[0], ptr_bytes[1], self.length as u8, 0x00]
    }
}

/// This is a custom circular buffer written for holding the messages
/// Consider making this Generic and generally just *better* and probably breaking out into it's own file maybe moving read functionality around a bit as well
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MessageBuffer {
    pub buffer: Vec<u8>, // we use Vecs since serde doesn't serialize / deserialize for arrays bigger than 32 elements long
    pub front_ptr: usize,
    pub length: usize,
    pub message_ptrs: Vec<MessagePointer>,
}

impl MessageBuffer {
    /// This is the number of messages that we can store in our buffer
    /// since we *currently* have a 512B buffer and each message is atleast 4 bytes that gives us a maximum of 128 messages
    const MESSAGE_PTRS_CAP: usize = 128;
    /// Size of the actual message data buffer  
    const BUFFER_CAP: usize = 512;
    /// Total size of the message buffer that the mmio can read from
    const MESSAGE_BUFFER_SIZE: usize =
        1 + MessageBuffer::BUFFER_CAP + MessageBuffer::MESSAGE_PTRS_CAP;

    pub fn new() -> Self {
        MessageBuffer {
            buffer: Vec::with_capacity(MessageBuffer::BUFFER_CAP),
            front_ptr: 0,
            length: 0,
            message_ptrs: Vec::with_capacity(MessageBuffer::MESSAGE_PTRS_CAP),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub fn is_full(&self) -> bool {
        self.available_space() < 4
    }

    pub fn available_space(&self) -> usize {
        let last: MessagePointer = self.message_ptrs
            [(self.front_ptr + self.length) % MessageBuffer::MESSAGE_PTRS_CAP];
        if (last.ptr + last.length) / MessageBuffer::BUFFER_CAP >= 1 {
            // if this is true the messages wrap around the end of the buffer
            // so the last message ends before the first one
            return self.message_ptrs[self.front_ptr].ptr
                - (last.ptr + last.length);
        }
        self.message_ptrs[self.front_ptr].ptr
            + (MessageBuffer::MESSAGE_PTRS_CAP - (last.ptr + last.length))
    }

    pub fn write(&mut self, v: &[u8]) -> Result<(), MessageBufferError> {
        // we should make it so messages can only start at every 4th byte so the
        // memory loads can always read from the start of a message, alternatively
        // write a more complicated read functionality
        if v.len() > self.available_space() {
            return Err(MessageBufferError::NoSpace);
        }
        let last_index =
            (self.front_ptr + self.length) % MessageBuffer::MESSAGE_PTRS_CAP;
        let last: MessagePointer = self.message_ptrs[last_index];
        let off_byte: usize = (4 - ((last.ptr + last.length) % 4)) % 4; // this is used to clamp the write to start at a multiple of four for easier reading
        let byte_index =
            (last.ptr + last.length + off_byte) % MessageBuffer::BUFFER_CAP;
        for (index, byte) in v.iter().enumerate() {
            self.buffer[(byte_index + index) % MessageBuffer::BUFFER_CAP] =
                *byte;
        }
        self.length += 1;
        self.message_ptrs[(last_index + 1) % MessageBuffer::MESSAGE_PTRS_CAP] =
            MessagePointer {
                ptr: byte_index,
                length: v.len(),
            };
        Ok(())
    }

    pub fn pop(&mut self) -> Result<(), MessageBufferError> {
        // Consider renaming as it isn't really a pop (it doesn't return anything)
        if self.is_empty() {
            return Err(MessageBufferError::NoMessage);
        }
        self.length -= 1;
        self.front_ptr = (self.front_ptr + 1) % MessageBuffer::MESSAGE_PTRS_CAP;
        Ok(())
    }
}

impl BotRadio {
    const SEND_BUFFER_SIZE: usize = 128;
    /// This function runs each tick!
    pub fn tick(&mut self) {
        self.cooldown = self.cooldown.saturating_sub(1);
    }

    /// This is the endpoint for reading from the bluetooth module
    /// Every increase in 1 by the rdi offset increases the the addr by 4
    /// | Addr beyond 7168 | What it returns |
    /// | --- | --- |
    /// | 0 | is ready to send a message? |
    /// | 1..128 | read the current message ready to send |
    /// | 128..769 | read the message buffer memory, the first 4 bytes are info on the buffer, the next 128 are the ptrs to the messages, the final 512 are the actual messages|
    ///  We currently don't read from the filter
    pub fn mmio_load(&self, addr: u32) -> Result<u32, ()> {
        match addr {
            AliveBot::MEM_RADIO => Ok(self.radio_status()),
            addr if addr >= AliveBot::MEM_RADIO + 4 => {
                let idx = (addr - AliveBot::MEM_RADIO - 4) as usize;
                // if it's the first 128 values read the send buffer
                match idx {
                    0..BotRadio::SEND_BUFFER_SIZE => self
                        .read_send_buffer(idx)
                        .map(u32::from_le_bytes)
                        .map_err(|_| ()),
                    BotRadio::SEND_BUFFER_SIZE
                        ..MessageBuffer::MESSAGE_BUFFER_SIZE => self
                        .read_message_buffer(idx - BotRadio::SEND_BUFFER_SIZE)
                        .map(u32::from_le_bytes)
                        .ok_or(()),
                    _ => Err(()), // make sure if they are reading beyond these bounds they might be reading a module that has a further on adddress
                }
            }
            _ => Err(()),
        }
    }

    fn radio_status(&self) -> u32 {
        // first bit is for if it's on which atm is always 1
        let mut out = 1u32;
        if self.cooldown == 0 {
            out += 2;
        }
        if !self.messages.read().unwrap().is_empty() {
            out += 4;
        }
        out
    }

    fn read_send_buffer(
        &self,
        addr: usize,
    ) -> Result<[u8; 4], TryFromSliceError> {
        <[u8; 4]>::try_from(&self.out_message[addr..addr + 4])
    }

    /// When reading from the recieved message buffer
    /// | Addr | What should be there |
    /// | --- | --- |
    /// | 0 | [front_ptr in ptrs buffer, number of messages in buffer, Memory offset from 0 for start of actual messages, Null] |
    /// | 1..129 | Message pointers circular buffer (start indexing from front_ptr) |
    /// | 130..642 | The actual messages information (find out where to index to and how long each message is from the ptrs buffer) |
    fn read_message_buffer(&self, addr: usize) -> Option<[u8; 4]> {
        #![allow(clippy::match_overlapping_arm)] // The match is checked so that the `..=MessageBuffer::MESSABE_BUFFER_SIZE` arm only takes values we need for it
        let messages = self.messages.try_read().unwrap();
        match addr {
            0 => Some([
                messages.front_ptr as u8,
                messages.length as u8,
                (MessageBuffer::MESSAGE_PTRS_CAP + 1) as u8,
                0x00,
            ]),
            1..=MessageBuffer::MESSAGE_PTRS_CAP => {
                Some(messages.message_ptrs[addr - 1].as_mmio_output())
            }
            ..=MessageBuffer::MESSAGE_BUFFER_SIZE => {
                let buff_ptr = addr - (MessageBuffer::MESSAGE_PTRS_CAP + 2);
                Some(
                    <[u8; 4]>::try_from(
                        &messages.buffer[buff_ptr..buff_ptr + 4],
                    )
                    .unwrap(),
                )
            }
            _ => None,
        }
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
        messages.write(message).map(|_| ()).map_err(|_| ())
    }

    /// Cleans up the receive buffer (move the pointers forward)
    fn remove_front(&self) -> Result<(), ()> {
        let mut messages = self.messages.write().unwrap();
        messages.pop().map_err(|_| ()) // messages are checked for being empty inside pop
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
            (AliveBot::MEM_RADIO, [0x04, _, _, _]) => self.remove_front(),
            (addr, bytes)
                if (AliveBot::MEM_RADIO + 4..=AliveBot::MEM_RADIO + 128) // check they are writing to the out_buffer
                    .contains(&addr) =>
            {
                let idx = (addr - (AliveBot::MEM_RADIO + 1)) as usize;
                for (i, byte) in bytes.iter().enumerate() {
                    self.out_message[idx + i] = *byte;
                }
                Ok(())
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
        let message = &self.out_message[0..message_length];
        ctxt.msgs.add_message(message, ctxt.pos, range.len());
        self.cooldown = range.cooldown(ctxt);
    }
}

impl Default for BotRadio {
    fn default() -> Self {
        Self {
            cooldown: 0,
            messages: Arc::new(RwLock::new(MessageBuffer::new())),
            out_message: Vec::with_capacity(128),
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

#[derive(Debug)]
pub enum MessageBufferError {
    NoSpace,
    NoMessage,
}

#[cfg(test)]
mod radio_tests {
    use std::num::NonZeroU64;

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn message_buffer_write() {
        let mut buff = MessageBuffer::new();
        let v: Vec<u8> = vec![1, 2, 3, 4, 5];
        buff.write(&v);
        // check the length is now 1
        assert_eq!(buff.length, 1, "Buffer isn't the correct length");
        // check the front value now has the length we set
        assert_eq!(
            buff.message_ptrs[buff.front_ptr].length,
            v.len(),
            "Writen message has changed length"
        );
    }

    #[test]
    fn message_buffer_read() {
        let mut buff = MessageBuffer::new();
        let v: Vec<u8> = vec![1, 2, 3, 4, 5];
        let v2: Vec<u8> = vec![4, 5, 6, 7, 8];
        buff.write(&v);
        buff.write(&v2);
        let addr = buff.message_ptrs[buff.front_ptr].ptr;
        let len = buff.message_ptrs[buff.front_ptr].length;

        let slice = &buff.buffer[addr..(addr + len)];
        assert_eq!(slice, &v, "Read message was different to saved one");
    }

    #[test]
    fn message_buffer_pop() {
        let mut buff = MessageBuffer::new();
        assert!(buff.pop().is_err(), "empty pop didn't return an error");
        let v: Vec<u8> = vec![1, 2, 3, 4, 5, 6];
        buff.write(&v);
        let front_ptr_local = buff.front_ptr;
        assert!(buff.pop().is_ok(), "buffer pop doesn't return ok");
        assert!(buff.is_empty(), "Buff isn't empty after pop");
        assert!(
            buff.front_ptr != front_ptr_local,
            "Front pointer has not been updated"
        );
    }

    #[test]
    fn message_buffer_available_space() {
        let mut buff = MessageBuffer::new();
        let v: Vec<u8> = vec![1, 2, 3, 4];
        let old_size = buff.available_space();
        buff.write(&v);
        assert!(
            old_size - v.len() == buff.available_space(),
            "Available space after writing 4 bytes is not correct"
        );
    }

    #[test]
    fn message_buffer_is_full() {
        let mut buff = MessageBuffer::new();
        assert!(!buff.is_full(), "Buffer reports full when it is empty");
        let v: Vec<u8> = vec![1; MessageBuffer::BUFFER_CAP];
        buff.write(&v);
        assert!(buff.is_full(), "Buffer doesn't report full when it is");
    }
}

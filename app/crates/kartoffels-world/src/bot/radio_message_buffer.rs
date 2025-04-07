use serde::{Deserialize, Serialize};

/// For dealing with message pointers in the [`MessageBuffer`]
/// just hides some logic away in a nice neat section
#[derive(Debug, Clone, Deserialize, Serialize, Copy, Default)]
pub struct MessagePointer {
    pub ptr: usize,
    pub length: usize,
}

impl MessagePointer {
    pub const POINTER_SIZE_BYTES: usize = 4;
    pub fn as_mmio_output(&self) -> [u8; 4] {
        let ptr_bytes = (self.ptr as u16).to_le_bytes(); // Since there are 512Bytes in the buffer the address for every message can't be stored in a u8, we instead have to cast it down to a u16 annoyingly we only need one bit of these extra 8 we use
        [ptr_bytes[0], ptr_bytes[1], self.length as u8, 0x00]
    }
}

#[derive(Debug)]
pub enum MessageBufferError {
    NoSpace,
    NoMessage,
}
/// This is a custom circular buffer written for holding the messages
/// Consider making this Generic and generally just *better*
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MessageBuffer {
    pub buffer: Vec<u8>, // we use Vecs since serde doesn't serialize / deserialize for arrays bigger than 32 elements long
    pub front_ptr: usize,
    pub length: usize,
    pub message_ptrs: Vec<MessagePointer>,
}

/// Two optional modes for implementing the reading from the message buffer
///
/// - The bot can read directly from the message buffer, this requires a huge
///   amount of space for addressing since we need space for every potential pointer
///   if each pointer itself is 4 bytes then there is an equivelent space for
///   addressing as actual message memeory, if each device has 1024B of addressable
///   memeory this limits us to just under 512B since the first byte is needed
///   for reading and writing data about the actual device itself
///   Alternatively we could limit the number of possible messages stored, meaning
///   you wouldn't be able to actually fill the data buffer with 4 byte messages
///   You could theoretically crunch the message pointer down to ~3 bytes, but as
///   rdi on the bot's side only allows for indexing at multiples of 4 this would
///   make reading from the ptrs buffer very complicated
///
/// - Only make the front message / a limited amount of the message buffer available
///   This would tread the radio module like an off-the-shelf component, this
///   makes the interface much easier to use, but limits the control a *power user*
///   might have. You can't compare two messages you've recieved without copying
///   them to some other location in memory, basically forcing / encouraging
///   the user to handle each message one at a time sequentially
///  
/// For convenience I've gone and implemented possible soltutions for both options
/// named:
/// - full_read (for being able to read every bit of data the message buffer holds)
/// - covered_read (for being able to only read the front of the buffer)
///
/// I'll currently plug in the second option
/// And my first option version assumes we can just use 1028B of addressable
/// space (actually 1160B since the actual first 4 bytes are info on the radio module
/// itself and the next 128 are the send buffer) though switching to a cap on the
/// number of possible messages we can store should be as easy as editing one const
/// and probably the available space.
impl MessageBuffer {
    // How many messages can we store (currently since the minimum size of a message is 4 bytes)
    const NUM_MESSAGES_CAN_STORE: usize = (MessageBuffer::BUFFER_CAP / 4); // edit this for full_read

    /// How many bytes take up the message pointers buffer
    pub const MESSAGE_PTRS_CAP: usize = MessageBuffer::NUM_MESSAGES_CAN_STORE
        * MessagePointer::POINTER_SIZE_BYTES;

    /// Size of the actual message data buffer  
    pub const BUFFER_CAP: usize = 512;

    /// Total size of the message buffer that the mmio can read from
    pub const MESSAGE_BUFFER_SIZE: usize =
        1 + MessageBuffer::BUFFER_CAP + MessageBuffer::MESSAGE_PTRS_CAP;

    pub fn new() -> Self {
        MessageBuffer {
            buffer: Vec::from([0; MessageBuffer::BUFFER_CAP]),
            front_ptr: 0,
            length: 0,
            message_ptrs: Vec::from(
                [MessagePointer::default(); MessageBuffer::MESSAGE_PTRS_CAP],
            ),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    #[allow(unused)]
    pub fn is_full(&self) -> bool {
        self.available_space() < 4
    }

    pub fn front_message_length(&self) -> u16 {
        if !self.is_empty() {
            self.message_ptrs[self.front_ptr].length as u16
        } else {
            0
        }
    }

    /// How many bytes available in the message buffer
    pub fn available_space(&self) -> usize {
        if self.is_empty() {
            return MessageBuffer::BUFFER_CAP;
        }
        let last: MessagePointer =
            self.message_ptrs[(self.front_ptr + self.length - 1)
                % MessageBuffer::MESSAGE_PTRS_CAP];

        if (last.ptr + last.length) / MessageBuffer::BUFFER_CAP >= 1 {
            // if this is true the messages wrap around the end of the buffer
            // so the last message ends before the first one
            return self.message_ptrs[self.front_ptr].ptr.saturating_sub(
                (last.ptr + last.length) % MessageBuffer::BUFFER_CAP,
            );
        }

        self.message_ptrs[self.front_ptr].ptr
            + (MessageBuffer::MESSAGE_PTRS_CAP - (last.ptr + last.length))
    }

    pub fn write(&mut self, v: &[u8]) -> Result<(), MessageBufferError> {
        if v.len() > self.available_space() {
            return Err(MessageBufferError::NoSpace);
        }

        let byte_index = if self.is_empty() {
            self.message_ptrs[self.front_ptr] = MessagePointer {
                ptr: 0,
                length: v.len(),
            };
            0
        } else {
            let last_index = (self.front_ptr + self.length - 1)
                % MessageBuffer::MESSAGE_PTRS_CAP;
            let last: MessagePointer = self.message_ptrs[last_index];
            let off_byte: usize = (4 - ((last.ptr + last.length) % 4)) % 4; // this is used to clamp the write to start at a multiple of four for easier reading
            let byte_index =
                (last.ptr + last.length + off_byte) % MessageBuffer::BUFFER_CAP;
            self.message_ptrs
                [(last_index + 1) % MessageBuffer::MESSAGE_PTRS_CAP] =
                MessagePointer {
                    ptr: byte_index,
                    length: v.len(),
                };
            byte_index
        };

        for (index, byte) in v.iter().enumerate() {
            self.buffer[(byte_index + index) % MessageBuffer::BUFFER_CAP] =
                *byte;
        }

        // Update the pointers
        self.length += 1;

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

    /// read from the struct in the format mmio_load will
    ///
    /// Both read functions should return None if addr is outside of their bounds,
    /// so that reading from further modules is doable
    #[allow(unused)]
    pub fn mmio_reader_full_read(&self, addr: usize) -> Option<[u8; 4]> {
        if addr == 0 {
            return Some([
                self.front_ptr as u8,
                self.length as u8,
                (MessageBuffer::MESSAGE_PTRS_CAP + 1) as u8,
                0x00,
            ]);
        }
        let data_addr = (addr / 4) - 1;
        match data_addr {
            0..MessageBuffer::MESSAGE_PTRS_CAP => {
                Some(self.message_ptrs[data_addr].as_mmio_output())
            }
            MessageBuffer::MESSAGE_PTRS_CAP
                ..MessageBuffer::MESSAGE_BUFFER_SIZE => {
                let buff_ptr = data_addr - MessageBuffer::MESSAGE_PTRS_CAP;
                Some(
                    <[u8; 4]>::try_from(&self.buffer[buff_ptr..buff_ptr + 4])
                        .unwrap(), // We can unwrap here as each addr in will be a multiple of 4
                )
            }
            _ => None,
        }
    }

    /// read from the struct in the format mmio_load will
    ///
    /// Both read functions should return None if addr is outside of their bounds,
    /// so that reading from further modules is doable
    pub fn mmio_reader_covered_read(&self, addr: usize) -> Option<[u8; 4]> {
        if addr >= 128 {
            return None;
        }
        let msg = self.message_ptrs[self.front_ptr];
        let msg_slice: &[u8] =
            if ((msg.ptr + msg.length) / MessageBuffer::BUFFER_CAP) >= 1 {
                // The message wraps around
                let next_len = msg.ptr + msg.length - MessageBuffer::BUFFER_CAP;
                &[&self.buffer[msg.ptr..], &self.buffer[..next_len]].concat()
            } else {
                &self.buffer[msg.ptr..msg.ptr + msg.length]
            };
        if addr + 4 < msg_slice.len() {
            Some(<[u8; 4]>::try_from(&msg_slice[addr..addr + 4]).unwrap())
        } else if addr < msg_slice.len() {
            let filler_slice: &[u8] =
                &vec![0; 4 - (msg_slice.len() - addr)][..];
            let combined: &[u8] = &[&msg_slice[addr..], filler_slice].concat();
            Some(<[u8; 4]>::try_from(combined).unwrap())
        } else {
            Some([0; 4])
        }
    }
}

#[cfg(test)]
mod radio_message_buffer_tests {

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn message_buffer_write() {
        let mut buff = MessageBuffer::new();
        let v: Vec<u8> = vec![1, 2, 3, 4, 5];
        let _ = buff.write(&v);
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
        let _ = buff.write(&v);
        let _ = buff.write(&v2);
        let len = buff.front_message_length() as usize;
        let mut front_msg: Vec<u8> = Vec::new();
        for i in 0..=(len / 4) {
            let read = buff.mmio_reader_covered_read(i * 4).unwrap_or([0; 4]);
            for v in read {
                front_msg.push(v);
            }
        }

        let slice = &front_msg[..len];
        assert_eq!(slice, &v, "Read message was different to saved one");
    }

    #[test]
    fn message_buffer_pop() {
        let mut buff = MessageBuffer::new();
        assert!(buff.pop().is_err(), "empty pop didn't return an error");
        let v: Vec<u8> = vec![1, 2, 3, 4, 5, 6];
        let _ = buff.write(&v);
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
        let _ = buff.write(&v);
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
        let _ = buff.write(&v);
        assert!(buff.is_full(), "Buffer doesn't report full when it is");
    }
}

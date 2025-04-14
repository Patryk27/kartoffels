use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum MessageBufferError {
    NoSpace,
    NoMessage,
    ReadOutOfBounds,
    IndexBeyondBounds,
    MessageDoesntExist,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MessageBuffer {
    /// Memory where the messages are stored, the first byte is the length of the first message, then followed by the next message
    /// This means indexing to an nth element is linear in time
    ///
    /// a message's maximum size is 128 bytes currently
    #[serde(with = "serde_bytes")]
    pub memory: [u8; MessageBuffer::RECIEVE_ADDRESS_SIZE],
    /// location of the first empty location in the memory (basically the 1 past the final stored message byte)
    pub back: usize,

    /// This is the buffer for the message that will be sent
    #[serde(with = "serde_bytes")]
    pub send_message: [u8; MessageBuffer::SEND_ADDRESS_SIZE],
}

impl MessageBuffer {
    /// Size of the recieved message buffer that the mmio can read from
    pub const RECIEVE_ADDRESS_SIZE: usize = 512;
    /// Size of the send message buffer that can be written and read to / from
    pub const SEND_ADDRESS_SIZE: usize = 128;
    /// Total size of the buffer
    pub const TOTAL_SIZE: usize =
        MessageBuffer::SEND_ADDRESS_SIZE + MessageBuffer::RECIEVE_ADDRESS_SIZE;

    pub fn new() -> Self {
        MessageBuffer {
            memory: [0; MessageBuffer::RECIEVE_ADDRESS_SIZE],
            back: 0,
            send_message: [0; MessageBuffer::SEND_ADDRESS_SIZE],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.front_message_length() == 0
    }

    #[allow(unused)]
    pub fn is_full(&self) -> bool {
        self.available_space() < 4
    }

    pub fn front_message_length(&self) -> u8 {
        self.memory[0]
    }

    /// How many bytes available in the message buffer
    pub fn available_space(&self) -> usize {
        if self.is_empty() {
            return MessageBuffer::RECIEVE_ADDRESS_SIZE;
        }
        self.memory.len() - self.back
    }

    pub fn write_to_recieve(
        &mut self,
        v: &[u8],
    ) -> Result<(), MessageBufferError> {
        if self.is_full() {
            return Err(MessageBufferError::NoSpace);
        }
        let space_left = self.available_space();
        if v.len() > space_left {
            self.memory[self.back] = (space_left - 1) as u8;
            self.memory[self.back + 1..].clone_from_slice(&v[..space_left - 1]);
            self.back = self.memory.len()
        } else {
            self.memory[self.back] = v.len() as u8;
            self.memory[self.back + 1..self.back + v.len() + 1]
                .copy_from_slice(v);
            self.back += v.len() + 1;
        }
        Ok(())
    }

    fn find_message_ptr(
        &self,
        index: usize,
    ) -> Result<usize, MessageBufferError> {
        if self.memory[0] == 0 {
            return Err(MessageBufferError::NoMessage);
        }
        let mut ptr = 0;
        for _ in 0..index {
            let Some(len) = self.memory.get(ptr) else {
                // we've read over the end of the buffer
                return Err(MessageBufferError::IndexBeyondBounds);
            };
            if *len == 0 {
                // we've read to the end of the messages
                return Err(MessageBufferError::MessageDoesntExist);
            }
            ptr += (*len + 1) as usize;
        }
        self.memory
            .get(ptr)
            .ok_or(MessageBufferError::MessageDoesntExist)
            .map(|_| ptr)
    }

    #[allow(unused)]
    pub fn read_recieve(
        &self,
        index: usize,
    ) -> Result<&[u8], MessageBufferError> {
        let ptr = self.find_message_ptr(index)?;
        let Some(msg_len) = self.memory.get(ptr) else {
            return Err(MessageBufferError::ReadOutOfBounds);
        };
        Ok(&self.memory[ptr + 1..ptr + 1 + (*msg_len as usize)])
    }

    pub fn mmio_read(&self, addr: usize) -> Result<[u8; 4], ()> {
        // bounds checking the read
        if addr > MessageBuffer::TOTAL_SIZE - 4 {
            return Err(());
        }
        match addr {
            ..MessageBuffer::SEND_ADDRESS_SIZE => {
                <[u8; 4]>::try_from(&self.send_message[addr..addr + 4])
                    .map_err(|_| ())
            }
            MessageBuffer::SEND_ADDRESS_SIZE.. => {
                let idx = addr - MessageBuffer::SEND_ADDRESS_SIZE;
                <[u8; 4]>::try_from(&self.memory[idx..idx + 4]).map_err(|_| ())
            }
        }
    }

    pub fn mmio_write(&mut self, addr: usize, data: &[u8]) -> Result<(), ()> {
        if addr > MessageBuffer::SEND_ADDRESS_SIZE - 4 {
            return Err(());
        }
        self.send_message[addr..addr + 4].copy_from_slice(data);
        Ok(())
    }

    pub fn remove_recieve(&mut self, index: usize) -> Result<(), ()> {
        // let's find this message and the following one
        let del_ptr: usize = self.find_message_ptr(index).map_err(|_| ())?;
        let del_len: usize = self.memory[del_ptr] as usize;
        let move_forward_ptr = del_ptr + 1 + self.memory[del_ptr] as usize;
        // check the following message exists (so we will need to pull stuff forward)
        let len = self.memory[move_forward_ptr];
        if len != 0 {
            // move all the data forwards over the message we are deleting
            self.memory
                .copy_within(move_forward_ptr..self.back, del_ptr); // now we need to clean everything from the back
            let new_back = self.back - (del_len + 1);
            self.memory[new_back..new_back + del_len + 1].fill(0);
            self.back = new_back;
        } else {
            // There is nothing after the message to be deleted so we will zero out the message and move the back pointer back
            self.memory[del_ptr..].fill(0); // consider replacing this also with a copy_within
            self.back = del_ptr;
        }
        Ok(())
    }

    #[allow(unused)]
    pub fn remove_recieve_front(&mut self) -> Result<(), ()> {
        // this might be faster if you can assume they are removing the front value, TODO: CHECK
        // check if empty
        if self.back == 0 {
            return Err(());
        }
        let len: usize = (self.memory[0] + 1) as usize;
        self.memory.copy_within(len.., 0);
        // move the back ptr back and clean up the moved back messages
        let new_back = self.back - len;
        self.memory[new_back..].fill(0);
        self.back = new_back;
        Ok(())
    }
}

#[cfg(test)]
mod radio_message_buffer_tests {

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn message_buffer_write() {
        let mut buff = MessageBuffer::new();
        let new_msg: Vec<u8> = vec![1, 2, 3, 4, 5, 6, 7, 8];
        assert!(buff.write_to_recieve(&new_msg[..]).is_ok());
        assert!(!buff.is_empty());

        // finally let's just do a bit more of a manual check that the write writes
        // the correct stuff this is a bit more implementation dependent so if you
        // change how it works on the backend expect this next section to fail
        // more often

        assert!(
            buff.memory[0] == new_msg.len() as u8,
            "Written message did not have the same logged space"
        );
        assert_eq!(
            &buff.memory[1..1 + new_msg.len()],
            &new_msg[..],
            "Written message did not have the same data"
        );
    }

    #[test]
    fn message_buffer_sequential_write() {
        // this is very implmentation dependent
        let mut buff = MessageBuffer::new();
        let msg_1: Vec<u8> = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let msg_2: Vec<u8> = vec![65, 67, 65, 66, 65, 67, 65, 66];
        assert!(buff.write_to_recieve(&msg_1[..]).is_ok());
        assert!(buff.write_to_recieve(&msg_2[..]).is_ok());

        // the data we should have stored is
        // [8,1,2,3,4,5,6,7,8,8,65,67,65,66,65,67,65,66]
        let test_out: Vec<u8> =
            vec![8, 1, 2, 3, 4, 5, 6, 7, 8, 8, 65, 67, 65, 66, 65, 67, 65, 66];
        assert_eq!(
            &buff.memory[0..test_out.len()],
            &test_out[..],
            "Two sequential writes did not store the correct data"
        );
    }

    #[test]
    fn message_buffer_remove_messages() {
        let mut buff = MessageBuffer::new();
        let msg_1: Vec<u8> = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let msg_2: Vec<u8> = vec![4, 3, 2, 1];
        let msg_3: Vec<u8> = vec![65, 67, 65, 66, 65, 67, 65, 66];
        let _ = buff.write_to_recieve(&msg_1[..]);
        let _ = buff.write_to_recieve(&msg_2[..]);
        let _ = buff.write_to_recieve(&msg_3[..]);

        // first try and delete the middle message
        assert!(buff.remove_recieve(1).is_ok());
        assert!(buff.remove_recieve(10).is_err()); // trying  to delete a message that isn't there should error
        let mut test_out: Vec<u8> = vec![
            8, 1, 2, 3, 4, 5, 6, 7, 8, 8, 65, 67, 65, 66, 65, 67, 65, 66, 0,
        ]; // the extra 0's make sure the previosu 3rd message was cleaned up
        assert_eq!(
            &buff.memory[0..test_out.len()],
            &test_out[..],
            "Deleting a middle message failed to create the right data"
        );
        // now let's try removing the front message
        assert!(buff.remove_recieve(0).is_ok());
        test_out = vec![8, 65, 67, 65, 66, 65, 67, 65, 66, 0];
        assert_eq!(
            &buff.memory[0..test_out.len()],
            &test_out[..],
            "Deleting a front message failed to create the right data"
        );

        assert!(buff.remove_recieve(0).is_ok()); // clean the buffer
        assert!(buff.remove_recieve(0).is_err()); // trying to delete from an empty buffer should error
    }

    #[test]
    fn message_buffer_available_space() {
        let mut buff = MessageBuffer::new();
        assert_eq!(buff.available_space(), MessageBuffer::RECIEVE_ADDRESS_SIZE);
        let new_msg: Vec<u8> = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let _ = buff.write_to_recieve(&new_msg[..]);

        assert_eq!(
            buff.available_space(),
            MessageBuffer::RECIEVE_ADDRESS_SIZE - (new_msg.len() + 1)
        ); // one message takes away the correct space`
    }

    #[test]
    fn message_buffer_is_full() {
        let mut buff = MessageBuffer::new();
        assert!(!buff.is_full()); // buff should NOT be full already!
        let filler_message: [u8; 100] = [10; 100];
        for _ in 0..6 {
            let _ = buff.write_to_recieve(&filler_message);
        }
        assert!(buff.is_full()) // buff should now be full
    }

    #[test]
    fn messsage_buffer_internal_read() {
        let mut buff = MessageBuffer::new();
        let msg_1: Vec<u8> = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let msg_2: Vec<u8> = vec![4, 3, 2, 1];
        let msg_3: Vec<u8> = vec![65, 67, 65, 66, 65, 67, 65, 66];
        let _ = buff.write_to_recieve(&msg_1[..]);
        let _ = buff.write_to_recieve(&msg_2[..]);
        let _ = buff.write_to_recieve(&msg_3[..]);
        let read_0 = buff.read_recieve(0);
        assert!(read_0.is_ok());
        assert_eq!(read_0.unwrap(), &msg_1[..]);
        let read_1 = buff.read_recieve(1);
        assert!(read_1.is_ok());
        assert_eq!(read_1.unwrap(), &msg_2[..]);
        let read_2 = buff.read_recieve(2);
        assert!(read_2.is_ok());
        assert_eq!(read_2.unwrap(), &msg_3[..]);
    }

    #[test]
    fn message_buffer_e2e() {
        // lots of sequential reads and writes and stuff to make sure that everything works
        let mut buff = MessageBuffer::new();
        assert!(buff.is_empty());
        let a_msg: Vec<u8> = vec![65, 65, 65, 65];
        let b_msg: Vec<u8> = vec![2; 128];
        let c_msg: Vec<u8> = vec![64; 64];
        // a list of sequential writes
        let message_writes: Vec<&Vec<u8>> =
            vec![&a_msg, &a_msg, &b_msg, &c_msg, &b_msg, &b_msg, &c_msg];
        (0..5).for_each(|i| {
            // write the first 5 elements
            assert!(buff.write_to_recieve(&(message_writes[i])[..]).is_ok());
            let read = buff.read_recieve(i);
            assert!(read.is_ok());
            assert_eq!(read.unwrap(), &message_writes[i][..]);
        });
        // let's then do a couple of deletes
        assert!(buff.remove_recieve(1).is_ok());
        assert!(buff.remove_recieve(0).is_ok());

        (2..5).for_each(|i| {
            let read = buff.read_recieve(i - 2);
            assert!(read.is_ok());
            assert_eq!(read.unwrap(), &message_writes[i][..]);
        });
    }

    #[test]
    fn message_buffer_mmio_read() {
        let mut buff = MessageBuffer::new();
        // set up some *data* in the send_buffer
        buff.send_message[..8].copy_from_slice(&[0, 1, 2, 3, 4, 5, 6, 7]);
        // set up some *data* in the recieve_buffer
        let msg_1: Vec<u8> = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let _ = buff.write_to_recieve(&msg_1);
        let mut read = buff.mmio_read(0);
        assert!(read.is_ok_and(|v| v == [0, 1, 2, 3]));
        read = buff.mmio_read(4);
        assert!(read.is_ok_and(|v| v == [4, 5, 6, 7]));
        read = buff.mmio_read(8);
        assert!(read.is_ok_and(|v| v == [0, 0, 0, 0]));
        // now lets read from the recieve buffer
        let mut ptr = MessageBuffer::SEND_ADDRESS_SIZE;
        read = buff.mmio_read(ptr);
        assert!(read.is_ok_and(|v| v == [8, 1, 2, 3]));
        ptr += 4;
        let read = buff.mmio_read(ptr);
        assert!(read.is_ok_and(|v| v == [4, 5, 6, 7]));
        ptr += 4;
        let read = buff.mmio_read(ptr);
        assert!(read.is_ok_and(|v| v == [8, 0, 0, 0]));
        assert!(buff.mmio_read(MessageBuffer::TOTAL_SIZE).is_err());
    }

    #[test]
    fn message_buffer_mmio_write() {
        let mut buff = MessageBuffer::new();
        assert!(buff.mmio_write(0, &[1, 2, 3, 4]).is_ok());
        assert_eq!(buff.send_message[..4], [1, 2, 3, 4]);
        assert!(buff
            .mmio_write(MessageBuffer::SEND_ADDRESS_SIZE, &[0, 0, 0, 0])
            .is_err());
    }
}

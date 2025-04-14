use core::cmp::min;

use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;

use crate::{cmd, rdi, wri, MEM_RADIO};

/// Reads the radio status address, this is a u32 which contains:
/// - if the radio is on
/// - can send a message
/// - has messages to read
/// - length of message to read
///
/// Other functions can unwrap this number into the actual info, see:
/// [`is_radio_ready()`], [`is_radio_on()`], [`radio_messages_to_read()`], [`radio_next_message_length()`]
pub fn radio_status() -> u32 {
    rdi(MEM_RADIO, 0)
}

/// Is the radio module turned on?
///
/// This is a feature that is yet to really be implemented to do with the battery, as such
/// the radio is at the moment always 1!
pub fn is_radio_on() -> bool {
    radio_status() & 1 != 0
}

/// Returns whether the radio sender is ready for [``] to run
///
/// See also: [`radio_wait()`]
pub fn is_radio_ready() -> bool {
    radio_status() & 2 != 0
}

/// Waits for the radio sender to be ready
///
/// See also: [`is_radio_ready()`]
pub fn radio_wait() {
    while !is_radio_ready() {
        //
    }
}

/// Are there any messages we can read from the radio's message buffer?
///
/// You can find the length of the message as well with [`radio_next_message_length()`]
/// and you can read the message with [`radio_read_next_message()`]
pub fn radio_messages_to_read() -> bool {
    radio_status() & 4 != 0
}

/// Whats the size of the next message we are ready to read
///
/// You can check if there is a message to read with [`radio_messages_to_read()`]
/// you can read the message with [`radio_read_next_message()`]
pub fn radio_next_message_length() -> usize {
    ((radio_status() >> 16) as u16) as usize
}

/// Write a message ready to be sent
pub fn radio_write_out_message(message: &[u8]) {
    for (index, chnk) in
        message[..min(message.len(), 128)].chunks(4).enumerate()
    {
        // there might be more optimal solutions (try coerce the slice to write and if that fails then do this code since that's only going to happen at most 1 time per call)
        let mut send_array: [u8; 4] = [0; 4];
        send_array[..chnk.len()].copy_from_slice(chnk);
        wri(MEM_RADIO, index + 1, <u32>::from_le_bytes(send_array))
    }
}

/// Read whatever message we are ready to send
/// Since you editing this won't actually edit the message we don't
/// really need mutability
pub fn radio_read_out_message() -> Arc<[u8]> {
    let mut out: Vec<u8> = Vec::with_capacity(128);
    for i in 0..32 {
        let mem_read: [u8; 4] = radio_read_data_at_index(i);
        out.extend_from_slice(&mem_read);
    }
    out.into()
}

/// An internal function for reading from the radio module
/// the first 32 reads the send message the next 128 are the message buffer
pub fn radio_read_data_at_index(index: usize) -> [u8; 4] {
    rdi(MEM_RADIO, 1 + index).to_le_bytes()
}

/// Read the first message in the buffer
pub fn radio_top_message() -> Option<Arc<[u8]>> {
    if !radio_messages_to_read() {
        return None;
    }
    let first_read: [u8; 4] = radio_read_data_at_index(32);
    let mut message_len = first_read[0] as usize;
    let mut out: Vec<u8> = Vec::with_capacity(message_len);
    out.extend_from_slice(&first_read[1..]);
    message_len -= 3;
    let mut counter = 1;
    while message_len > 0 {
        let data = radio_read_data_at_index(32 + counter);
        out.extend_from_slice(&data[..min(message_len, 4)]);
        message_len = message_len.saturating_sub(4);
        counter += 1;
    }
    Some(out.into())
}

// Reads the nth message in the buffer, if no message is there returns a None!
//
// This is quite a large function, might consider breaking it up and cleaning it a lot
pub fn radio_read_nth_message(index: usize) -> Option<Arc<[u8]>> {
    if !radio_messages_to_read() {
        return None;
    }
    let mut target_index = 0;
    let mut counter = 0;
    while target_index < 128 + 32 {
        if counter == index {
            let first_read = radio_read_data_at_index(32 + (target_index / 4));
            let internal_pos = target_index % 4;
            let mut message_len = first_read[internal_pos] as usize;
            let mut out: Vec<u8> = Vec::with_capacity(message_len);
            out.extend_from_slice(&first_read[internal_pos..]);
            message_len -= 3 - (internal_pos);
            let mut inner_counter = (target_index / 4) + 1;
            while message_len > 0 {
                let data = radio_read_data_at_index(32 + inner_counter);
                out.extend_from_slice(&data[..min(message_len, 4)]);
                message_len = message_len.saturating_sub(4);
                inner_counter += 1;
            }
            return Some(out.into());
        }
        let data = radio_read_data_at_index(32 + (target_index / 4));
        target_index += match data[target_index % 4] as usize {
            0 => return None,
            v => v + 1,
        };
        counter += 1;
    }
    None
}

/// Turn radio on
pub fn radio_turn_on() {
    wri(MEM_RADIO, 0, cmd(0x01, 0, 0, 0));
}

/// Internal function for sending radio messages valid powers are 3,5,7,9;
pub fn radio_send_signal(length: u8, power: u8) {
    wri(MEM_RADIO, 0, cmd(0x02, length, power, 0));
}

pub fn radio_set_filter(filter: [u8; 4]) {
    wri(MEM_RADIO, 0, cmd(0x03, 0, filter[0], filter[1]));
    wri(MEM_RADIO, 0, cmd(0x03, 1, filter[2], filter[3]));
}

pub fn radio_stop_filter() {
    wri(MEM_RADIO, 0, cmd(0x03, 2, 0, 0));
}

pub fn radio_delete_message(index: usize) {
    wri(MEM_RADIO, 0, cmd(0x04, index as u8, 0, 0));
}

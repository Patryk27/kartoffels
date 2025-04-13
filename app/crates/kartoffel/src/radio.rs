use core::num::NonZeroU64;

use alloc::vec::Vec;

use crate::{cmd, rdi, wri, MEM_RADIO};

/// Reads the radio status address, this is a u32 which contains:
/// - if the radio is on
/// - can send a message
/// - has messages to read
/// - length of message to read
/// all baked into it
/// Other functions can unwrap this number into the actual info, see:
/// [`is_radio_ready()`], [`is_radio_on()`], [`radio_messages_to_read()`], [`radio_next_message_length()`]
#[inline(always)]
pub fn radio_status() -> u32 {
    rdi(MEM_RADIO, 0)
}

/// Is the radio module turned on?
///
/// This is a feature that is yet to really be implemented to do with the battery, as such
/// the radio is at the moment always 1!
#[inline(always)]
pub fn is_radio_on() -> bool {
    radio_status() & 1 != 0
}

/// Returns whether the radio sender is ready for [``] to run
///
/// See also: [`radio_wait()`]
#[inline(always)]
pub fn is_radio_ready() -> bool {
    radio_status() & 2 != 0
}

/// Waits for the radio sender to be ready
///
/// See also: [`is_radio_ready()`]
#[inline(always)]
pub fn radio_wait() {
    while !is_radio_ready() {
        //
    }
}

/// Are there any messages we can read from the radio's message buffer?
///
/// You can find the length of the message as well with [`radio_next_message_length()`]
/// and you can read the message with [`radio_read_next_message()`]
#[inline(always)]
pub fn radio_messages_to_read() -> bool {
    radio_status() & 4 != 0
}

/// Whats the size of the next message we are ready to read
///
/// You can check if there is a message to read with [`radio_messages_to_read()`]
/// you can read the message with [`radio_read_next_message()`]
#[inline(always)]
pub fn radio_next_message_length() -> usize {
    let bytes: [u8; 2] =
        <[u8; 2]>::try_from(&radio_status().to_le_bytes()[2..]).unwrap();
    u16::from_le_bytes(bytes) as usize
}

/// Read the current message we are preparing to send
#[inline(always)]
pub fn radio_read_out_message() -> Vec<u8> {
    let mut out: Vec<u8> = Vec::with_capacity(128);
    for off in 1..(128 / 4) {
        let message_bytes: [u8; 4] = rdi(MEM_RADIO, off).to_le_bytes();
        out.extend_from_slice(&message_bytes[..]);
    }
    out
}

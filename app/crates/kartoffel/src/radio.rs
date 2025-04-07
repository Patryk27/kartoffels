use core::num::NonZeroU64;

use crate::{cmd, rdi, wri, MEM_RADIO};

#[derive(Debug)]
pub struct Message {
    pub message: [u8; 32],
    pub sender_id: NonZeroU64,
}

/// Returns whether the bluetooth sender is ready for [`send_bluetooth()`] to run
///
/// See also: [`bluetooth_wait()`]
#[inline(always)]
pub fn is_bluetooth_ready() -> bool {
    rdi(MEM_RADIO, 0) == 1
}

/// Waits for the bluetooth sender to be ready
///
/// See also: [`is_bluetooth_ready()`]
#[inline(always)]
pub fn bluetooth_wait() {
    while !is_bluetooth_ready() {
        //
    }
}

/// Sends whatever is currently in the send buffer in a `r x r` range
/// Only values of 3, 5, 7, or 9 will work for r
///
/// Alternatively use the easier to use functions:
///
/// - [`bluetooth_send_3x3()`]
/// - [`bluetooth_send_5x5()`]
/// - [`bluetooth_send_7x7()`]
/// - [`bluetooth_send_9x9()`]
#[inline(always)]
pub fn send_bluetooth(r: u8) {
    wri(MEM_RADIO, 0, cmd(0x01, r, 0x00, 0x00));
}

// these functions need better names
/// Overwrite the entire send buffer, this won't **send** the message just prepares the one that will be sent
///
/// to do that See: [`send_bluetooth()`]
#[inline(always)]
pub fn bluetooth_write_send_buffer(message: [u8; 32]) {
    for (i, byte) in message.iter().enumerate() {
        wri(MEM_RADIO, 0, cmd(0x02, i as u8, *byte, 0x00));
    }
}

/// Overwrites a single byte in the send buffer, this is slow and generally superseded by [`bluetooth_write_send_buffer()`]
///
/// See: [`send_bluetooth()`] [`bluetooth_write_send_buffer()`]
#[inline(always)]
pub fn bluetooth_write_send_buffer_byte(data: u8, addr: usize) {
    if addr >= 32 {
        return;
    }
    wri(MEM_RADIO, 0, cmd(0x02, addr as u8, data, 0x00));
}

/// This sets the bluetooth send buffer to [0;32] cleaning everything out so if you don't write over everything there is no reminants of the old sent message
#[inline(always)]
pub fn clear_bluetooth_send_buffer() {
    wri(MEM_RADIO, 0, cmd(0x03, 0x00, 0x00, 0x00));
}

/// This is used to move the front point ahead one and reduce the length by 1 in a circular buffer this effectively means removing the front (or oldest) item
/// This is mostly used as a low level function and you should instead use:
///
/// - [`bluetooth_pop_read_buffer()`]
/// - [`bluetooth_peek_read_buffer()`]
#[inline(always)]
pub fn bluetooth_remove_read_buffer_front() {
    wri(MEM_RADIO, 0, cmd(0x04, 0x00, 0x00, 0x00));
}

/// Read the front (oldest) item in your read buffer and clean up the space making it available to be written to
/// if you want to be able to read it again, consider using [`bluetooth_peek_read_buffer()`].
///
/// TODO Example
pub fn bluetooth_pop_read_buffer() -> Option<Message> {
    let [front, _, empty, bytes_per] = rdi(MEM_RADIO, 1).to_le_bytes();
    if empty == 1 {
        return None;
    }
    let start_byte = 1 + ((front * bytes_per) / 4);
    let message =
        read_message_from_offset(start_byte as usize, bytes_per as usize);
    bluetooth_remove_read_buffer_front();
    Some(message)
}

/// Read the front (oldest) item in your read buffer, but leave it there for reading later:
/// if you would like to get rid of the message either call:
/// - [`bluetooth_pop_read_buffer()`]
/// - [`bluetooth_remove_read_buffer_front()`]
///
/// TODO Example
#[inline(always)]
pub fn bluetooth_peek_read_buffer() -> Option<Message> {
    // let's get the important info here
    let [front, _, empty, bytes_per] = rdi(MEM_RADIO, 1).to_le_bytes();
    if empty == 1 {
        return None;
    }
    let start_byte = 1 + ((front * bytes_per) / 4);
    Some(read_message_from_offset(
        start_byte as usize,
        bytes_per as usize,
    ))
}

/// This reads a message from the buffer at the given index
/// The message buffer is a maximum of 5 long (thought you might not have that many messages ready to read)
/// to check the current length of the buffer see:
/// - [`bluetooth_read_buffer_length()`]
///
/// TODO Example
#[inline(always)]
pub fn bluetooth_get_from_read_buffer(index: usize) -> Option<Message> {
    let [front, length, empty, bytes_per] = rdi(MEM_RADIO, 1).to_le_bytes();
    if empty == 1 || index > (length - 1) as usize {
        return None;
    }
    let start_byte = 1 + (((front + index as u8) % 5 * bytes_per) / 4);
    Some(read_message_from_offset(
        start_byte as usize,
        bytes_per as usize,
    ))
}

/// reads a message into a Message object
/// This is a low-level function - for convenience you'll probably want to use:
/// - [`bluetooth_peek_read_buffer()`]
/// - [`bluetooth_pop_read_buffer()`]
#[inline(always)]
fn read_message_from_offset(start_byte: usize, bytes_per: usize) -> Message {
    let mut out: [u8; 32] = [0; 32];
    let mut d1: u64 = 0;
    let mut d2: u64 = 0;
    for off in 0..(bytes_per / 4) {
        // this is hard coded atm to the data in kartoffels_world/bot/bluetooth.rs, maybe we should serde this or something
        match off {
            0..=7 => {
                let byte_group =
                    rdi(MEM_RADIO, 1 + start_byte + off).to_le_bytes();
                (0..4).for_each(|v| out[(off * 4) + v] = byte_group[v])
            }
            8 => {
                d1 = rdi(MEM_RADIO, 1 + start_byte + off) as u64;
            }
            9 => {
                d2 = rdi(MEM_RADIO, 1 + start_byte + off) as u64;
            }
            _ => {}
        }
    }
    Message {
        message: out,
        sender_id: NonZeroU64::new((d1 << 32) | d2).unwrap(),
    }
}

/// Is there anything in the read buffer
#[inline(always)]
pub fn bluetooth_is_read_buffer_empty() -> bool {
    let [_, _, is_empty, _] = rdi(MEM_RADIO, 1).to_le_bytes();
    is_empty != 0
}

/// How many elements in the read buffer, useful for: [`bluetooth_get_from_read_buffer()`]
pub fn bluetooth_read_buffer_length() -> u8 {
    let [_, l, _, _] = rdi(MEM_RADIO, 1).to_le_bytes();
    l
}

/// Sends a message to bots in a 3x3 area:
///
/// # Cooldown
///
/// ```text
/// 20_000 +- 10% ticks (~300ms)
/// ```
///
/// # Example
/// ```no_run
/// # use kartoffel::*;
/// #
///
/// let message: [u8;32]  = "Po-ta-toes! Boil 'em, mash 'em!!".chars().map(|v| v as u8).collect::<Vec<u8>>().try_into().unwrap();
/// bluetooth_wait();
/// bluetooth_send_3x3(message);
///
/// // We might not have received any messages, but if we have received one, lets read it!
/// if let Some(message) = bluetooth_pop_read_buffer() {
///     let string_from_message: String = message.message.iter().map(|&c| c as char).collect()
///     println!("{}" string_from_message);
/// }
/// ```
#[inline(always)]
pub fn bluetooth_send_3x3(message: [u8; 32]) {
    bluetooth_write_send_buffer(message);
    send_bluetooth(3);
    clear_bluetooth_send_buffer();
}

/// Sends a message to bots in a 5x5 area:
///
/// # Cooldown
///
/// ```text
/// 23_000 +- 15% ticks (~)
/// ```
///
/// # Example
/// ```no_run
/// # use kartoffel::*;
/// #
///
/// let message: [u8;32]  = "Po-ta-toes! Boil 'em, mash 'em!!".chars().map(|v| v as u8).collect::<Vec<u8>>().try_into().unwrap();
/// bluetooth_wait();
/// bluetooth_send_5x5(message);
///
/// // We might not have received any messages, but if we have received one, lets read it!
/// if let Some(message) = bluetooth_pop_read_buffer() {
///     let string_from_message: String = message.message.iter().map(|&c| c as char).collect()
///     println!("{}" string_from_message);
/// }
/// ```
#[inline(always)]
pub fn bluetooth_send_5x5(message: [u8; 32]) {
    bluetooth_write_send_buffer(message);
    send_bluetooth(5);
    clear_bluetooth_send_buffer();
}

/// Sends a message to bots in a 7x7 area:
///
/// # Cooldown
///
/// ```text
/// 28_000 +- 25% ticks (~)
/// ```
///
/// # Example
/// ```no_run
/// # use kartoffel::*;
/// #
///
/// let message: [u8;32]  = "Po-ta-toes! Boil 'em, mash 'em!!".chars().map(|v| v as u8).collect::<Vec<u8>>().try_into().unwrap();
/// bluetooth_wait();
/// bluetooth_send_7x7(message);
///
/// // We might not have received any messages, but if we have received one, lets read it!
/// if let Some(message) = bluetooth_pop_read_buffer() {
///     let string_from_message: String = message.message.iter().map(|&c| c as char).collect()
///     println!("{}" string_from_message);
/// }
/// ```
#[inline(always)]
pub fn bluetooth_send_7x7(message: [u8; 32]) {
    bluetooth_write_send_buffer(message);
    send_bluetooth(7);
    clear_bluetooth_send_buffer();
}

/// Sends a message to bots in a 9x9 area:
///
/// # Cooldown
///
/// ```text
/// 35_000 +- 30% ticks (~)
/// ```
///
/// # Example
/// ```no_run
/// # use kartoffel::*;
/// #
///
/// let message: [u8;32]  = "Po-ta-toes! Boil 'em, mash 'em!!".chars().map(|v| v as u8).collect::<Vec<u8>>().try_into().unwrap();
/// bluetooth_wait();
/// bluetooth_send_9x9(message);
///
/// // We might not have received any messages, but if we have received one, lets read it!
/// if let Some(message) = bluetooth_pop_read_buffer() {
///     let string_from_message: String = message.message.iter().map(|&c| c as char).collect()
///     println!("{}" string_from_message);
/// }
/// ```
#[inline(always)]
pub fn bluetooth_send_9x9(message: [u8; 32]) {
    bluetooth_write_send_buffer(message);
    send_bluetooth(9);
    clear_bluetooth_send_buffer();
}

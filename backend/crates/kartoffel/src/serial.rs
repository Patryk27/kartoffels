use crate::{wri, MEM_SERIAL};

/// Sends a single character to the serial port where it can be read by the
/// user (within the web browser).
///
/// Serial port is a circular buffer with capacity for 256 UTF-8 characters, so
/// writing 257th character will shift all characters by one, removing the first
/// character.
#[inline(always)]
pub fn serial_send(ch: char) {
    wri(MEM_SERIAL, 0, ch as u32);
}

/// Sends a string to the serial port.
///
/// See: [`serial_send()`].
#[inline(always)]
pub fn serial_send_str(str: &str) {
    for ch in str.chars() {
        serial_send(ch);
    }
}

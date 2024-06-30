use crate::{wri, MEM_SERIAL};

/// Sends a character to the serial port (i.e. the web browser).
///
/// Serial port is a circular buffer with capacity for 256 UTF-8 characters -
/// writing 257th character will shift all the characters by one, removing the
/// first character.
///
/// See also: [`serial_send_str()`], [`serial_send_ansi()`].
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

/// Sends a special control character - see: [`SerialCtrlChar`].
///
/// See also: [`serial_send()`], [`serial_send_str()`].
///
/// # Examples
///
/// ## Reducing flickering
///
/// If you plan on displaying something animated etc., the web terminal might
/// flicker - you can get rid of this artifact like so:
///
/// ```rust,no_run
/// # use kartoffel::*;
/// #
/// let mut n = 0;
///
/// loop {
///     serial_send_ctrl(SerialCtrlChar::StartBuffering);
///     serial_send_str("hello: ");
///     serial_send((b'0' + (n % 10)) as char);
///     serial_send_ctrl(SerialCtrlChar::FlushBuffer);
/// }
/// ```
pub fn serial_send_ctrl(ctrl: SerialCtrlChar) {
    wri(MEM_SERIAL, 0, ctrl.encode());
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SerialCtrlChar {
    /// Start buffering the output.
    ///
    /// All characters sent from this point on will not be displayed until you
    /// send [`SerialCtrlChar::FlushBuffer`].
    StartBuffering,

    /// Flush the buffered output and print it on the terminal.
    FlushBuffer,
}

impl SerialCtrlChar {
    pub fn encode(&self) -> u32 {
        let ctrl = match self {
            SerialCtrlChar::StartBuffering => 0x00,
            SerialCtrlChar::FlushBuffer => 0x01,
        };

        0xffffff00 | ctrl
    }
}

use crate::{wri, MEM_SERIAL};
use alloc::string::String;

/// Writes a character or a string to the serial port.
///
/// Serial port is a circular buffer with capacity for 256 UTF-8 characters -
/// writing 257th character will shift all the characters by one, removing the
/// first character.
///
/// See also: [`SerialControlCode`].
///
/// # Example
///
/// ```no_run
/// # use kartoffel::*;
/// #
/// serial_write('H');
/// serial_write("Hello\nWorld");
/// serial_write(format!("n = {}", 123));
/// ```
#[inline(always)]
pub fn serial_write(val: impl SerialWritable) {
    val.write();
}

/// Terminal control code - similar to ANSI color code, i.e. it allows to
/// manipulate the terminal.
///
/// # Examples
///
/// ## Reducing flickering
///
/// If you plan on displaying something animated, the terminal might flicker -
/// you can get rid of this like so:
///
/// ```rust,no_run
/// # use kartoffel::*;
/// #
/// let mut n = 0;
///
/// loop {
///     serial_write(SerialControlCode::StartBuffering);
///     serial_write("hello: ");
///     serial_write(format!("{n}"));
///     serial_write(SerialControlCode::FlushBuffer);
/// }
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SerialControlCode {
    /// Start buffering the output.
    ///
    /// All characters sent from this point on will not be displayed until you
    /// send [`SerialControlCode::FlushBuffer`].
    StartBuffering,

    /// Flush the buffered output and print it on the terminal.
    FlushBuffer,
}

impl SerialControlCode {
    pub fn encode(&self) -> u32 {
        let ctrl = match self {
            SerialControlCode::StartBuffering => 0x00,
            SerialControlCode::FlushBuffer => 0x01,
        };

        0xffffff00 | ctrl
    }
}

/// Thing that can be written into the terminal - see [`serial_write()`].
pub trait SerialWritable {
    fn write(self);
}

impl SerialWritable for char {
    fn write(self) {
        wri(MEM_SERIAL, 0, self as u32);
    }
}

impl SerialWritable for &str {
    fn write(self) {
        for ch in self.chars() {
            ch.write();
        }
    }
}

impl SerialWritable for String {
    fn write(self) {
        for ch in self.chars() {
            ch.write();
        }
    }
}

impl SerialWritable for SerialControlCode {
    fn write(self) {
        wri(MEM_SERIAL, 0, self.encode());
    }
}

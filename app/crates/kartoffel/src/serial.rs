use crate::*;
use core::fmt;

/// Writes a single character to the serial port.
///
/// Serial port is a circular buffer with capacity for 256 UTF-8 characters -
/// writing 257th character shifts all the previous characters, removing the
/// first one.
///
/// This is a low-level function - for convenience you'll most likely want to
/// use [`print!()`](crate::print!()) or [`println!()`](crate::println!()).
///
/// See also: [`serial_buffer()`].
///
/// # Example
///
/// ```no_run
/// # use kartoffel::{*, println};
/// #
/// serial_write('H');
/// serial_write('e');
/// serial_write('l');
/// serial_write('l');
/// serial_write('o');
/// serial_write('!');
/// serial_write('\n');
///
/// // or:
///
/// println!("Hello!");
/// println!("Hello, {}!", "World");
/// ```
pub fn serial_write(ch: char) {
    unsafe {
        wri(MEM_SERIAL, 0, ch as u32);
    }
}

/// Enables buffering.
///
/// In this mode all characters written into the serial port get buffered until
/// you call [`serial_flush()`] or [`serial_clear()`].
///
/// This comes handy for animations, interactive UIs etc., since it prevents
/// seeing partially written text (also known as _tearing_).
///
/// # Example
///
/// ```no_run
/// # use kartoffel::{*, println};
/// #
/// loop {
///     serial_buffer();
///
///     println!("Hello, World!");
///     println!("ticks = {}", timer_ticks());
///
///     serial_flush();
/// }
/// ```
pub fn serial_buffer() {
    unsafe {
        wri(MEM_SERIAL, 0, 0xffffff00);
    }
}

/// Flushes buffered characters.
///
/// If buffering hasn't been enabled ([`serial_buffer()`]), this function does
/// nothing.
pub fn serial_flush() {
    unsafe {
        wri(MEM_SERIAL, 0, 0xffffff01);
    }
}

/// Clears buffered characters.
///
/// If buffering hasn't been enabled ([`serial_buffer()`]), this function does
/// nothing.
pub fn serial_clear() {
    unsafe {
        wri(MEM_SERIAL, 0, 0xffffff02);
    }
}

/// Allows to `write!()` and `writeln!()` into the serial port.
///
/// See also: [`print!()`](crate::print!()), [`println!()`](crate::println!()).
///
/// # Example
///
/// ```no_run
/// # use kartoffel::{*, println};
/// use core::fmt::Write;
///
/// writeln!(&mut Serial, "Hello!").unwrap();
/// writeln!(&mut Serial, "Hello, {}!", "World").unwrap();
///
/// // or just:
///
/// println!("Hello!");
/// println!("Hello, {}!", "World");
/// ```
pub struct Serial;

impl fmt::Write for Serial {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_char(c)?;
        }

        Ok(())
    }

    fn write_char(&mut self, c: char) -> fmt::Result {
        serial_write(c);

        Ok(())
    }
}

/// Prints to the serial port.
///
/// See also: [`serial_write()`].
///
/// # Example
///
/// ```no_run
/// # use kartoffel::print;
/// #
/// print!("Hello");
/// print!("{}!", "World");
/// print!("\n");
/// ```
#[macro_export]
macro_rules! print {
    ($($t:tt)*) => {{
        use ::core::fmt::Write;

        write!($crate::Serial, $($t)*).unwrap();
    }};
}

/// Prints to the serial port, with a newline.
///
/// See also: [`serial_write()`].
///
/// # Example
///
/// ```no_run
/// # use kartoffel::println;
/// #
/// println!("Hello!");
/// println!("Hello, {}!", "World");
/// ```
#[macro_export]
macro_rules! println {
    ($($t:tt)*) => {{
        use ::core::fmt::Write;

        writeln!($crate::Serial, $($t)*).unwrap();
    }};
}

use crate::*;

#[doc(hidden)]
pub const COMPASS_MEM: u32 = MEM + 6 * 1024;

/// Interrupt raised when compass has a new measurement.
///
/// See: [`irq_set()`], [`compass_dir()`].
///
/// # Example
///
/// ```no_run
/// # use kartoffel::{*, println};
/// #
/// fn on_compass_ready(args: u32) {
///     match args.to_le_bytes()[1] {
///         1 => println!("north"),
///         2 => println!("east"),
///         3 => println!("south"),
///         4 => println!("west"),
///         _ => unreachable!(),
///     }
/// }
///
/// irq_set(IRQ_COMPASS_READY, irq!(on_compass_ready));
/// ```
pub const IRQ_COMPASS_READY: u8 = 9;

/// Returns which direction the bot was facing at the time of the latest
/// measurement and then removes this measurement from the compass.
///
/// The first measurement is available immediately after the bot is born, and it
/// corresponds to the bot's direction right after spawning, while the next
/// measurements are provided automatically every ~128k ticks (~2s).
///
/// Calling this function before the next measurement is ready returns zero,
/// indicating `measurement not ready yet`.
///
/// # Cooldown
///
/// - ~128k ticks (~2s).
///
/// # Interrupts
///
/// - [`IRQ_COMPASS_READY`].
///
/// # Example
///
/// ```no_run
/// # use kartoffel::{*, println};
/// #
/// match compass_dir() {
///     0 => println!("measurement not ready yet"),
///     1 => println!("north"),
///     2 => println!("east"),
///     3 => println!("south"),
///     4 => println!("west"),
///     _ => unreachable!(),
/// }
/// ```
///
pub fn compass_dir() -> u32 {
    unsafe { rdi(COMPASS_MEM, 0) }
}

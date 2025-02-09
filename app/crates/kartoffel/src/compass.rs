use crate::{rdi, MEM_COMPASS};

/// Returns which direction the bot was facing at the time of the latest
/// measurement, and then removes that measurement from the compass.
///
/// The first measurement is available immediately after the bot is born, and it
/// corresponds to the bot's direction right after spawning, while the next
/// measurements are provided automatically every ~128k ticks (~2s).
///
/// Calling this function before the next measurement is ready returns zero,
/// indicating `measurement not ready yet`.
///
/// # Example
///
/// ```no_run
/// # use kartoffel::*;
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
pub fn compass_dir() -> u32 {
    rdi(MEM_COMPASS, 0)
}

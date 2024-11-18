use crate::{rdi, MEM_COMPASS};

/// Returns which direction the robot was facing at the time of the latest
/// measurement, and then removes that measurement from the compass.
///
/// The first measurement is available immediately after the bot is spawned, and
/// it corresponds to the robot's direction right after spawning, while the next
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
///     0 => serial_write("measurement not ready yet"),
///     1 => serial_write("north"),
///     2 => serial_write("east"),
///     3 => serial_write("south"),
///     4 => serial_write("west"),
///     _ => unreachable!(),
/// }
/// ```
pub fn compass_dir() -> u32 {
    rdi(MEM_COMPASS, 0)
}

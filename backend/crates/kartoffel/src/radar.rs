use crate::{rdi, wri, MEM_RADAR};
use core::array;

/// Returns whether the radar is ready and [`radar_scan()`] can be invoked.
///
/// See: [`radar_wait()`].
#[inline(always)]
pub fn is_radar_ready() -> bool {
    rdi(MEM_RADAR, 0) == 1
}

/// Waits for the radar to become ready.
///
/// See: [`is_radar_ready()`].
#[inline(always)]
pub fn radar_wait() {
    while !is_radar_ready() {
        //
    }
}

/// Scans a `d` x `d` square around the bot and returns the scan as a 2D array.
///
/// Possible values of `d` are `3`, `5`, `7` and `9`.
///
/// # Example
///
/// If the map contained the following tiles:
///
/// ```text
/// A B C
/// D @ F
/// G H I
/// ```
///
/// ... then calling `radar_scan::<3>()` would return an YX-indexed array of:
///
/// ```text
/// [
///   ['A', 'B', 'C'],
///   ['D', '@', 'F'],
///   ['G', 'H', 'I']
/// ]
/// ```
///
/// So, for instance, for `D = 3`:
///
/// - `arr[1][1]` is always the center tile (so the bot itself),
/// - `arr[0][1]` is always the tile in front of us,
/// - `arr[2][1]` is always the tile behind us,
/// - `arr[1][0]` is always the tile to our left,
/// - `arr[1][2]` is always the tile to our right.
///
/// For `D = 5`, the center tile would be at `arr[2][2]` etc.
///
/// # Cooldown
///
/// This function introduces a cooldown that depends on `d`:
///
/// - given `d = 3`, the cooldown is 10_000 +- 10% ticks (~150 ms),
/// - given `d = 5`, the cooldown is 15_000 +- 15% ticks (~230 ms),
/// - given `d = 7`, the cooldown is 22_000 +- 25% ticks (~310 ms),
/// - given `d = 9`, the cooldown is 30_000 +- 30% ticks (~460 ms).
#[inline(always)]
pub fn radar_scan<const D: usize>() -> [[char; D]; D] {
    wri(MEM_RADAR, 0, D as u32);

    array::from_fn(|y| {
        array::from_fn(|x| rdi(MEM_RADAR, y * D + x + 1) as u8 as char)
    })
}

use crate::{rdi, wri, MEM_RADAR};
use core::array;

/// Returns whether the radar is ready and [`radar_scan()`] or [`radar_read()`]
/// can be invoked.
#[inline(always)]
pub fn is_radar_ready() -> bool {
    rdi(MEM_RADAR, 0) == 1
}

#[inline(always)]
pub fn radar_wait() {
    while !is_radar_ready() {
        //
    }
}

/// Scans a `d * d` square of tiles around the bot; possible values of `d` are
/// `3`, `5`, `7` and `9`.
///
/// After calling this function, wait until [`is_radar_ready()`] returns `true`
/// and then use [`radar_read()`] to get the results.
///
/// # Example
///
/// Calling `radar_scan(3)` will scan a 3x3 square:
///
/// ```text
/// . . .
/// . @ .
/// . . .
/// ```
#[inline(always)]
pub fn radar_scan(d: u32) {
    wri(MEM_RADAR, 0, d);
}

/// Returns the result of the latest radar's scan, as a 2D yx-indexed array.
///
/// Const parameter `D` must match the number passed to [`radar_scan()`],
/// otherwise the results will be garbled.
///
/// # Example
///
/// If the map said:
///
/// ```text
/// A B C
/// D @ F
/// G H I
/// ```
///
/// ... then calling `radar_scan(3)` and then `radar_read::<3>()` would return:
///
/// ```text
/// [
///   ['A', 'B', 'C'],
///   ['D', '@', 'F'],
///   ['G', 'H', 'I']
/// ]
/// ```
///
/// ... so, when `D = 3`, then:
///
/// - `arr[1][1]` is always the center tile (us),
/// - `arr[0][1]` is always the tile in front of us,
/// - `arr[2][1]` is always the tile behind us,
/// - `arr[1][0]` is always the tile to our left,
/// - `arr[1][2]` is always the tile to our right.
///
/// When requesting with `D = 5`, the bot would be at `arr[2][2]` etc.
#[inline(always)]
pub fn radar_read<const D: usize>() -> [[char; D]; D] {
    array::from_fn(|y| {
        array::from_fn(|x| rdi(MEM_RADAR, y * D + x + 1) as u8 as char)
    })
}

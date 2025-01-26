use crate::{rdi, wri, MEM_RADAR};
use core::num::NonZeroU64;

/// Returns whether the radar is ready and [`radar_scan()`] can be invoked.
///
/// See also: [`radar_wait()`].
#[inline(always)]
pub fn is_radar_ready() -> bool {
    rdi(MEM_RADAR, 0) == 1
}

/// Waits for the radar to become ready.
///
/// See also: [`is_radar_ready()`].
#[inline(always)]
pub fn radar_wait() {
    while !is_radar_ready() {
        //
    }
}

/// Scans a square around the bot.
///
/// Note that this is a low-level function - for convenience you'll most likely
/// want to use one of:
///
/// - [`radar_scan_3x3()`],
/// - [`radar_scan_5x5()`],
/// - [`radar_scan_7x7()`],
/// - [`radar_scan_9x9()`].
///
/// See those functions for example usage.
///
/// # Arguments
///
/// Valid values of `r` are `3`, `5`, `7` or `9`. Calling this function with an
/// invalid value of `r` does nothing.
///
/// # Output
///
/// Use [`radar_read()`] to access the scanned data.
#[inline(always)]
pub fn radar_scan(r: usize) {
    wri(MEM_RADAR, 0, r as u32);
}

/// Scans a 3x3 square around the bot and returns the scanned area.
///
/// # Cooldown
///
/// ```text
/// 10_000 +- 10% ticks (~150 ms)
/// ```
///
/// # Example
///
/// ```no_run
/// # use kartoffel::*;
/// #
/// radar_wait();
///
/// let scan = radar_scan_3x3();
///
/// // If there's a bot in front of us, stab it
/// if scan.at(0, -1) == '@' && is_arm_ready() {
///     arm_stab();
/// }
/// ```
#[inline(always)]
pub fn radar_scan_3x3() -> RadarScan<3> {
    radar_scan(3);

    RadarScan { _priv: () }
}

/// Scans a 5x5 square around the bot and returns the scanned area.
///
/// # Cooldown
///
/// ```text
/// 15_000 +- 15% ticks (~230 ms)
/// ```
///
/// # Example
///
/// ```no_run
/// # use kartoffel::*;
/// #
/// radar_wait();
///
/// let scan = radar_scan_5x5();
///
/// // If there's a bot in front of us, stab it
/// if scan.at(0, -1) == '@' && is_arm_ready() {
///     arm_stab();
/// }
/// ```
#[inline(always)]
pub fn radar_scan_5x5() -> RadarScan<5> {
    radar_scan(5);

    RadarScan { _priv: () }
}

/// Scans a 7x7 square around the bot and returns the scanned area.
///
/// # Cooldown
///
/// ```text
/// 22_000 +- 25% ticks (~310 ms)
/// ```
///
/// # Example
///
/// ```no_run
/// # use kartoffel::*;
/// #
/// radar_wait();
///
/// let scan = radar_scan_7x7();
///
/// // If there's a bot in front of us, stab it
/// if scan.at(0, -1) == '@' && is_arm_ready() {
///     arm_stab();
/// }
/// ```
#[inline(always)]
pub fn radar_scan_7x7() -> RadarScan<7> {
    radar_scan(7);

    RadarScan { _priv: () }
}

/// Scans a 9x9 square around the bot and returns the scanned area.
///
/// # Cooldown
///
/// ```text
/// 30_000 +- 30% ticks (~460 ms)
/// ```
///
/// # Example
///
/// ```no_run
/// # use kartoffel::*;
/// #
/// radar_wait();
///
/// let scan = radar_scan_9x9();
///
/// // If there's a bot in front of us, stab it
/// if scan.at(0, -1) == '@' && is_arm_ready() {
///     arm_stab();
/// }
/// ```
#[inline(always)]
pub fn radar_scan_9x9() -> RadarScan<9> {
    radar_scan(9);

    RadarScan { _priv: () }
}

/// Reads data from the radar.
///
/// Note that this is a low-level function - for convenience you'll most likely
/// want to use [`RadarScan`], e.g. through [`radar_scan_3x3()`].
///
/// # Coordinate system
///
/// See: [`RadarScan`].
///
/// # Return value
///
/// Meaning of the returned number depends on `z`:
///
/// - `z=0` returns the tile located at `dx,dy` (see: [`RadarScan::at()`]),
///
/// - `z=1` returns the higher 32 bits of the id of the bot located at `dx,dy`
///   (see: [`RadarScan::bot_at()`]),
///
/// - `z=2` returns the lower 32 bits of the id of the bot located at `dx,dy`
///   (see: [`RadarScan::bot_at()`]).
pub fn radar_read(r: usize, dx: i8, dy: i8, z: u8) -> u32 {
    let x = (dx + (r as i8 / 2)) as usize;
    let y = (dy + (r as i8 / 2)) as usize;
    let z = z as usize;

    rdi(MEM_RADAR, 1 + z * r * r + y * r + x)
}

/// Outcome of a radar scan such as [`radar_scan_3x3()`].
///
/// # Coordinate system
///
/// [`Self::at()`] and [`Self::bot_at()`] work in bot-centric coordinate
/// system, that is:
///
/// - `.at(0, 0)` returns the bot itself (`'@'`),
/// - `.at(-1, 0)` returns tile to the left of the bot,
/// - `.at(1, 0)` returns tile to the right of the bot,
/// - `.at(0, -1)` returns tile in the front of the bot,
/// - `.at(0, 1)` returns tile in the back of the bot,
/// - etc.
///
/// This also means that the 3x3 scan allows you to access `at(-1..=1)`, 5x5
/// gives you `at(-2..=2)` etc.
///
/// The same applies to [`Self::bot_at()`].
///
/// # Lazyness
///
/// For performance reasons, this structure doesn't copy the scanned area into
/// your robot's RAM - rather, the data is kept inside the radar's memory and
/// transparently accessed each time you call [`Self::at()`] etc.
///
/// In practice, this means that consecutive scans *overwrite* previous results,
/// like:
///
/// ```no_run
/// # use kartoffel::*;
/// #
/// radar_wait();
///
/// let scan1 = radar_scan_5x5();
///
/// motor_wait();
/// motor_step();
/// radar_wait();
///
/// let scan2 = radar_scan_5x5();
///
/// // whoopsie, scan1 will return the same data as scan2!
/// // (i.e. it will return the newest scan)
/// ```
///
/// If you need to preserve an older scan, you should call [`Self::at()`] etc.
/// and store all the information you need elsewhere.
#[derive(Debug)]
pub struct RadarScan<const R: usize> {
    _priv: (),
}

impl<const R: usize> RadarScan<R> {
    /// Returns the topmost thing visible at given coordinates:
    ///
    /// - if there's a bot, returns `'@'`,
    /// - otherwise, if there's an object, returns that object (e.g. `'*'`),
    /// - otherwise, if there's a tile, returns that tile (e.g. `'.'` or `'|'`),
    /// - otherwise returns `' '` (a space) representing void (driving into it
    ///   makes you fall out of the map and die).
    ///
    /// # Coordinate system
    ///
    /// This function uses bot-centric coordinates, i.e. `at(0, -1)` points at
    /// the tile right in front of you - see [`RadarScan`] for details.
    pub fn at(&self, dx: i8, dy: i8) -> char {
        radar_read(R, dx, dy, 0) as u8 as char
    }

    /// Returns id of the bot at given coordinates or `None` if there's no bot
    /// there.
    ///
    /// Bot ids are random, unique, non-zero 64-bit numbers assigned to each bot
    /// during its upload; ids are preserved when a bot is reincarnated.
    ///
    /// # Coordinate system
    ///
    /// This function uses bot-centric coordinates, i.e. `bot_at(0, -1)` points
    /// at the bot right in front of you - see [`RadarScan`] for details.
    pub fn bot_at(&self, dx: i8, dy: i8) -> Option<NonZeroU64> {
        let d1 = radar_read(R, dx, dy, 1) as u64;
        let d2 = radar_read(R, dx, dy, 2) as u64;

        NonZeroU64::new((d1 << 32) | d2)
    }
}

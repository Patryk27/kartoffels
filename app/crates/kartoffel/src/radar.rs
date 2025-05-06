use crate::*;

#[doc(hidden)]
pub const RADAR_MEM: u32 = MEM + 5 * 1024;

/// Causes radar to look for surrounding tiles, such as floor (`.`) or walls
/// (`#`).
///
/// This option increases cooldown by 4k ticks.
///
/// This option is enabled by default - use [`radar_scan_ex()`] to disable it.
pub const RADAR_SCAN_TILES: u8 = 1 << 0;

/// Causes radar to look for surrounding bots (`@`).
///
/// This option increases cooldown by 4k ticks.
///
/// This option is enabled by default - use [`radar_scan_ex()`] to disable it.
pub const RADAR_SCAN_BOTS: u8 = 1 << 1;

/// Causes radar to look for surrounding objects, such as diamonds (`*`).
///
/// This option increases cooldown by 4k ticks.
///
/// This option is enabled by default - use [`radar_scan_ex()`] to disable it.
pub const RADAR_SCAN_OBJS: u8 = 1 << 2;

/// Causes radar to look for ids of surrounding things.
///
/// See: [`radar_read_id()`].
///
/// This option increases cooldown by 8k ticks.
///
/// This option is disabled by default - use [`radar_scan_ex()`] to activate it.
pub const RADAR_SCAN_IDS: u8 = 1 << 3;

/// Causes radar to look for directions the surrounding things are facing.
///
/// See: [`radar_read_dir()`].
///
/// This option increases cooldown by 8k ticks.
///
/// This option is disabled by default - use [`radar_scan_ex()`] to activate it.
pub const RADAR_SCAN_DIRS: u8 = 1 << 4;

/// Interrupt raised when radar becomes busy.
///
/// Note that this interrupt is raised only when radar _becomes_ busy - if radar
/// was already busy when you ran a command, the interrupt will not be rasied.
///
/// See: [`irq_set()`], [`IRQ_RADAR_IDLE`].
///
/// # Example
///
/// ```no_run
/// # use kartoffel::{*, println};
/// #
/// fn on_radar_busy() {
///     println!("radar busy");
/// }
///
/// fn on_radar_idle() {
///     println!("radar idle");
/// }
///
/// irq_set(IRQ_RADAR_BUSY, irq!(on_radar_busy));
/// irq_set(IRQ_RADAR_IDLE, irq!(on_radar_idle));
///
/// radar_scan(3);
/// ```
pub const IRQ_RADAR_BUSY: u8 = 7;

/// Interrupt raised when radar becomes idle.
///
/// See: [`irq_set()`], [`IRQ_RADAR_BUSY`].
///
/// # Example
///
/// ```no_run
/// # use kartoffel::{*, println};
/// #
/// fn on_radar_busy() {
///     println!("radar busy");
/// }
///
/// fn on_radar_idle() {
///     println!("radar idle");
/// }
///
/// irq_set(IRQ_RADAR_BUSY, irq!(on_radar_busy));
/// irq_set(IRQ_RADAR_IDLE, irq!(on_radar_idle));
///
/// radar_scan(3);
/// ```
pub const IRQ_RADAR_IDLE: u8 = 8;

/// Returns whether radar is ready.
///
/// See: [`radar_wait()`], [`IRQ_RADAR_BUSY`], [`IRQ_RADAR_IDLE`].
///
/// # Example
///
/// ```no_run
/// # use kartoffel::*;
/// #
/// if radar_ready() {
///     radar_scan(3);
/// }
/// ```
pub fn radar_ready() -> bool {
    unsafe { rdi(RADAR_MEM, 0) == 1 }
}

/// Waits until radar is ready.
///
/// See: [`radar_ready()`], [`IRQ_RADAR_BUSY`], [`IRQ_RADAR_IDLE`].
///
/// # Example
///
/// ```no_run
/// # use kartoffel::*;
/// #
/// radar_wait();
/// radar_scan(3);
///
/// // If there's someone in front of us, stab them
/// if radar_read(0, -1) == '@' && arm_ready() {
///     arm_stab();
/// }
/// ```
pub fn radar_wait() {
    while !radar_ready() {
        //
    }
}

/// Scans a square around the bot.
///
/// `range` defines the scanned square's length, e.g. `range=3` does a 3x3 scan;
/// legal values are 3, 5, 7 or 9, other ranges will cause the firmware to
/// crash.
///
/// See also [`radar_scan_ex()`], which allows you to specify custom scanning
/// options such as [`RADAR_SCAN_IDS`].
///
/// # Cooldown
///
/// - 3x3 scan = ~16k ticks (~250 ms)
/// - 5x5 scan = ~20k ticks (~312 ms)
/// - 7x7 scan = ~28k ticks (~437 ms)
/// - 9x9 scan = ~44k ticks (~678 ms)
///
/// # Example
///
/// ```no_run
/// # use kartoffel::*;
/// #
/// radar_wait();
/// radar_scan(3);
///
/// // If there's someone in front of us, stab them
/// if radar_read(0, -1) == '@' && arm_ready() {
///     arm_stab();
/// }
/// ```
pub fn radar_scan(range: u8) {
    radar_scan_ex(range, 0);
}

/// Scans a square around the bot, with custom options.
///
/// `range` defines the scanned square's length, as in [`radar_scan()`].
///
/// `opts` defines the scanning options, such as [`RADAR_SCAN_IDS`]; this is a
/// bitmask.
///
/// This function is an advanced variant of [`radar_scan()`] - it allows you to
/// perform either a faster scan or a more comprehensive one, see the examples
/// below.
///
/// Note that for backward-compatibility reasons, `opts = 0` is equivalent to
/// the default options:
///
/// ```no_run
/// # use kartoffel::*;
/// # let range = todo!();
/// #
/// radar_wait();
/// radar_scan_ex(range, 0);
///
/// // ^ is equivalent to:
/// radar_wait();
/// radar_scan_ex(range, RADAR_SCAN_TILES | RADAR_SCAN_BOTS | RADAR_SCAN_OBJS);
/// ```
///
/// # Cooldown
///
/// - 3x3 scan = ~4k ticks (~62 ms)
/// - 5x5 scan = ~8k ticks (~125 ms)
/// - 7x7 scan = ~16k ticks (~250 ms)
/// - 9x9 scan = ~32k ticks (~500 ms)
///
/// ... plus sum of the cooldowns of the options you picked.
///
/// # Examples
///
/// If you don't care about scanning tiles, you just want to know whether
/// there's a bot around you, you can call this function with `opts` equal to
/// [`RADAR_SCAN_BOTS`] - this will be faster than calling `radar_scan(3)`, but
/// it will return only information about the surrounding bots:
///
/// ```no_run
/// # use kartoffel::*;
/// #
/// radar_wait();
/// radar_scan_ex(3, RADAR_SCAN_BOTS);
///
/// // If there's someone in front of us, stab them
/// if radar_read(0, -1) == '@' && arm_ready() {
///     arm_stab();
/// }
/// ```
///
/// Or perhaps you need more information, in which case you can bundle a couple
/// of flags together - this will take longer than `radar_scan(3)`, but it will
/// also provide you more information:
///
/// ```no_run
/// # use kartoffel::*;
/// #
/// radar_wait();
///
/// radar_scan_ex(
///     3,
///     RADAR_SCAN_TILES
///     | RADAR_SCAN_BOTS
///     | RADAR_SCAN_OBJS
///     | RADAR_SCAN_IDS
///     | RADAR_SCAN_DIRS,
/// );
///
/// // If there's someone in front of us and it's looking at us, stab them
/// if radar_read(0, -1) == '@'
///     && radar_read_dir(0, -1) == 'v'
///     && arm_ready()
/// {
///     arm_stab();
/// }
/// ```
pub fn radar_scan_ex(range: u8, opts: u8) {
    // There are multiple ways of converting scan's two-dimensional indices,
    // such as `(-1, -3)` or `(0, 0)`, into one-dimensional memory addresses.
    //
    // Up until v0.7, we've used the typical:
    //
    //     idx = y * range + x
    //
    // ... which proved to be a bit awkward, because it requires for all reading
    // functions to be aware of the scan's range as well, so you'd do:
    //
    //     radar_scan(3);
    //
    // ... and then you'd carry around the range with you:
    //
    //     radar_read(3, /* ... */);
    //
    // Since v0.8 we use a new addressing mode, one that utilizes Szudzik's
    // pairing function and thus has stable indices - indices that are
    // irrespective of the scan's size, and this is what this argument enables.
    let addr = 0x01;

    unsafe {
        wri(RADAR_MEM, 0, pack(0x01, range, opts, addr));
    }
}

/// Returns type of topmost thing visible at given coordinates.
///
/// - if there's a bot, returns `'@'`,
/// - otherwise, if there's an object, returns that object (e.g. `'*'`),
/// - otherwise, if there's a tile, returns that tile (e.g. `'.'`),
/// - otherwise returns `' '` (a space) representing void (driving into it
///   makes you fall out of the map and die).
///
/// Basically, it returns you the same character you see on the screen.
///
/// # Requirements
///
/// Using this function requires for the scan to be performed with at least
/// one of the [`RADAR_SCAN_TILES`], [`RADAR_SCAN_BOTS`] or
/// [`RADAR_SCAN_OBJS`] option enabled, otherwise this function will return
/// zero.
///
/// Those options are enabled by default when you call [`radar_scan()`].
///
/// # Coordinate system
///
/// This function works in bot-centric coordinate system, that is:
///
/// - `radar_read(0, 0)` returns you,
/// - `radar_read(0, -1)` returns whatever is in front of you,
/// - `radar_read(0, 1)` returns whatever is behind you,
/// - `radar_read(-1, 0)` returns whatever is to your left,
/// - `radar_read(1, 0)` returns whatever is to your right,
/// - etc.
pub fn radar_read(x: i32, y: i32) -> char {
    radar_read_ex(x, y, 0).to_le_bytes()[0] as char
}

/// Returns id of the topmost thing visible at given coordinates.
///
/// Calling this function makes sense only for inspecting bots (`@`) or objects
/// (e.g. `*`), for other kinds of things the returned value will be zero.
///
/// # Example
///
/// ```no_run
/// # use kartoffel::{*, print};
/// #
/// radar_wait();
/// radar_scan_ex(3, RADAR_SCAN_BOTS | RADAR_SCAN_IDS);
///
/// if radar_read(0, -1) == '@' {
///     print!("looking at you, {}", radar_read_id(0, -1));
/// }
/// ```
///
/// # Requirements
///
/// Using this function requires performing scan with the [`RADAR_SCAN_IDS`]
/// option, which is disabled by default - see [`radar_scan_ex()`].
///
/// # Coordinate system
///
/// See [`radar_read()`].
pub fn radar_read_id(x: i32, y: i32) -> u64 {
    let hi = radar_read_ex(x, y, 1) as u64;
    let lo = radar_read_ex(x, y, 2) as u64;

    (hi << 32) | lo
}

/// Returns direction of the topmost thing visible at given coordinates.
///
/// Calling this function makes sense only for inspecting bots (`@`), for
/// other kinds of things the returned value will be zero.
///
/// You can't inspect your own direction, use [`compass_dir()`] for that.
///
/// # Example
///
/// ```no_run
/// # use kartoffel::{*, print};
/// #
/// radar_wait();
/// radar_scan_ex(3, RADAR_SCAN_BOTS | RADAR_SCAN_DIRS);
///
/// if radar_read(0, -1) == '@' && radar_read_dir(0, -1) == 'v' {
///     print!("someone's watching me");
/// }
/// ```
///
/// # Return value
///
/// One of `'<'`, `'^'`, `'>'` or `'v'`.
///
/// # Requirements
///
/// Using this function requires performing scan with the [`RADAR_SCAN_DIRS`]
/// option, which is disabled by default - see [`radar_scan_ex()`].
///
/// # Coordinate system
///
/// See [`radar_read()`].
pub fn radar_read_dir(x: i32, y: i32) -> char {
    radar_read_ex(x, y, 0).to_le_bytes()[1] as char
}

/// Reads data from radar.
///
/// This is a low-level function - for convenience you'll most likely want to
/// use [`radar_read()`], [`radar_read_id()`] etc.
///
/// Meaning of the returned value depends on `z`, inspect [`radar_read()`],
/// [`radar_read_dir()`] etc. for reference.
///
/// # Coordinate system
///
/// See [`radar_read()`].
pub fn radar_read_ex(x: i32, y: i32, z: u8) -> u32 {
    unsafe { rdi(RADAR_MEM, (1 + radar_idx(x, y, z)) as usize) }
}

/// Maps given coordinates into an index which you can use to access radar's
/// memory.
///
/// This is a low-level function - for convenience you'll most likely want to
/// use [`radar_read()`], [`radar_read_id()`] etc.
///
/// Note that while `x` and `y` are allowed to be arbitrary numbers, `z` must
/// lie within `0..=2`.
pub fn radar_idx(x: i32, y: i32, z: u8) -> i32 {
    // We're using Szudzik's pairing function, see:
    //
    // - https://www.vertexfragment.com/ramblings/cantor-szudzik-pairing-functions/
    // - http://szudzik.com/ElegantPairing.pdf

    let x = if x >= 0 { 2 * x } else { -2 * x - 1 };
    let y = if y >= 0 { 2 * y } else { -2 * y - 1 };
    let xy = if x >= y { x * x + x + y } else { y * y + x };

    3 * xy + (z as i32)
}

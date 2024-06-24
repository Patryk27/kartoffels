#![no_std]

use core::{array, ptr};

const MEM: *mut u32 = 0x08000000 as *mut u32;
const MEM_TIMER: *mut u32 = MEM;
const MEM_BATTERY: *mut u32 = MEM.wrapping_byte_add(1024);
const MEM_SERIAL: *mut u32 = MEM.wrapping_byte_add(2 * 1024);
const MEM_MOTOR: *mut u32 = MEM.wrapping_byte_add(3 * 1024);
const MEM_ARM: *mut u32 = MEM.wrapping_byte_add(4 * 1024);
const MEM_RADAR: *mut u32 = MEM.wrapping_byte_add(5 * 1024);

/// Returns a pseudorandom number that can be used as a source of randomness
/// for hashmaps etc.
///
/// Note that this doesn't return a *new* random number each time it's called -
/// rather the number is randomized once, when the bot is being (re)started.
#[inline(always)]
pub fn timer_seed() -> u32 {
    rdi(MEM_TIMER, 0)
}

/// Returns the number of ticks that have passed since the bot's started
/// working.
#[inline(always)]
pub fn timer_ticks() -> u32 {
    rdi(MEM_TIMER, 1)
}

/// Returns the remaining battery energy.
///
/// Since battery is not simulated at the moment, this function doesn't come
/// useful.
#[doc(hidden)]
#[inline(always)]
pub fn battery_energy() -> u32 {
    rdi(MEM_BATTERY, 0)
}

/// Sends a single character to the serial port where it can be read by the
/// user (within the web browser).
///
/// Serial port is a circular buffer with capacity for 256 UTF-8 characters, so
/// writing 257th character will shift all characters by one, removing the first
/// character.
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

/// Returns whether the motor is ready and [`motor_step()`] or [`motor_turn()`]
/// can be invoked.
#[inline(always)]
pub fn is_motor_ready() -> bool {
    rdi(MEM_MOTOR, 0) == 1
}

/// Moves bot one tile forward in the direction it's currently facing.
///
/// Note that this function has a cooldown period of 15000 ticks, see:
/// [`is_motor_ready()`].
#[inline(always)]
pub fn motor_step() {
    wri(MEM_MOTOR, 0, 1);
}

/// Turns bot:
///
/// - if angle is < 0, counterclockwise (i.e. "to left"),
/// - if angle is > 0, clockwise (i.e. "to right"),
/// - if angle is = 0, does nothing.
///
/// Only the sign of `angle` matters, i.e. `motor_turn(-123)` is the same as
/// just `motor_turn(-1)`.
///
/// Note that this function has a cooldown period of 10000 ticks, see:
/// [`is_motor_ready()`].
#[inline(always)]
pub fn motor_turn(angle: i32) {
    wri(MEM_MOTOR, 1, angle as u32);
}

/// Returns whether the arm is ready and [`arm_stab()`] can be invoked.
#[inline(always)]
pub fn is_arm_ready() -> bool {
    rdi(MEM_ARM, 0) == 1
}

/// Stabs the bot in front of us (if any), killing it.
///
/// Note that this function has a cooldown period of 15000 ticks, see:
/// [`is_arm_ready()`].
#[inline(always)]
pub fn arm_stab() {
    wri(MEM_ARM, 0, 1)
}

/// Returns whether the radar is ready and [`radar_scan()`] or [`radar_read()`]
/// can be invoked.
#[inline(always)]
pub fn is_radar_ready() -> bool {
    rdi(MEM_RADAR, 0) == 1
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

#[inline(always)]
fn rdi(ptr: *mut u32, off: usize) -> u32 {
    unsafe { ptr::read_volatile(ptr.wrapping_add(off)) }
}

#[inline(always)]
fn wri(ptr: *mut u32, off: usize, val: u32) {
    unsafe {
        ptr::write_volatile(ptr.wrapping_add(off), val);
    }
}

#[cfg(target_arch = "riscv64")]
#[macro_export]
macro_rules! init {
    (@ $(,)?) => {
        //
    };

    (@ +alloc($size:literal) $(, $($tt:tt)*)?) => {
        use spin::Mutex;
        use talc::*;
        use core::ptr::addr_of;

        static mut ARENA: [u8; $size] = [0; $size];

        #[global_allocator]
        static ALLOCATOR: Talck<Mutex<()>, ClaimOnOom> = Talc::new(unsafe {
            ClaimOnOom::new(Span::from_const_array(addr_of!(ARENA)))
        })
        .lock();

        $crate::init!(@ $($($tt)*)?);
    };

    (@ +panic $(, $($tt:tt)*)?) => {
        #[panic_handler]
        fn panic(_info: &core::panic::PanicInfo) -> ! {
            loop {}
        }

        $crate::init!(@ $($($tt)*)?);
    };

    ($($tt:tt)*) => {
        core::arch::global_asm!(
            r#"
            .global _start

            _start:
                la sp, _stack_start
                jal {}
                ebreak
            "#,
            sym main
        );

        $crate::init!(@ $($tt)*);
    };
}

#[cfg(not(target_arch = "riscv64"))]
#[macro_export]
macro_rules! init {
    ($($tt:tt)*) => {
        //
    };
}

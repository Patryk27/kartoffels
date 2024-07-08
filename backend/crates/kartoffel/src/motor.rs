use crate::{rdi, wri, MEM_MOTOR};

/// Returns whether the motor is ready and [`motor_step()`] or [`motor_turn()`]
/// can be invoked.
///
/// See: [`motor_wait()`].
#[inline(always)]
pub fn is_motor_ready() -> bool {
    rdi(MEM_MOTOR, 0) == 1
}

/// Waits for the motor to become ready.
///
/// See: [`is_motor_ready()`].
#[inline(always)]
pub fn motor_wait() {
    while !is_motor_ready() {
        //
    }
}

/// Moves the bot one tile forward in the direction it's currently facing.
///
/// # Cooldown
///
/// This function introduces a cooldown of 20_000 +- 15% ticks (~310 ms) - see:
/// [`is_motor_ready()`].
#[inline(always)]
pub fn motor_step() {
    wri(MEM_MOTOR, 0, 1);
}

/// Turns the bot in place:
///
/// - given angle < 0, counterclockwise ("to left"),
/// - given angle > 0, clockwise ("to right"),
/// - given angle = 0, does nothing.
///
/// Only the sign of `angle` matters, i.e. `motor_turn(-123)` is the same as
/// just `motor_turn(-1)`.
///
/// # Cooldown
///
/// This function introduces a cooldown of 10_000 +- 15% ticks (~150 ms) - see:
/// [`is_motor_ready()`].
#[inline(always)]
pub fn motor_turn(angle: i32) {
    wri(MEM_MOTOR, 1, angle as u32);
}

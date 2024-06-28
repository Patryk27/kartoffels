use crate::{rdi, wri, MEM_MOTOR};

/// Returns whether the motor is ready and [`motor_step()`] or [`motor_turn()`]
/// can be invoked.
#[inline(always)]
pub fn is_motor_ready() -> bool {
    rdi(MEM_MOTOR, 0) == 1
}

#[inline(always)]
pub fn motor_wait() {
    while !is_motor_ready() {
        //
    }
}

/// Moves bot one tile forward in the direction it's currently facing.
///
/// Note that this function has a cooldown period of 15000 ticks, see:
/// [`is_motor_ready()`].
#[inline(always)]
pub fn motor_step() {
    wri(MEM_MOTOR, 0, 1);
}

/// Turns bot in place (so it rotates the bot, but doesn't cause it to move
/// forward):
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

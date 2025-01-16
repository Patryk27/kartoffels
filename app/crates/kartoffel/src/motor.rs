use crate::{rdi, wri, MEM_MOTOR};

/// Returns whether the motor is ready and [`motor_step()`] or [`motor_turn()`]
/// can be invoked.
///
/// See also: [`motor_wait()`].
///
/// # Example
///
/// ```no_run
/// # use kartoffel::*;
/// #
/// loop {
///     if is_motor_ready() {
///         motor_wait();
///         break;
///     } else {
///         // do something else while we're waiting
///     }
/// }
/// ```
#[inline(always)]
pub fn is_motor_ready() -> bool {
    rdi(MEM_MOTOR, 0) == 1
}

/// Waits for the motor to become ready.
///
/// See also: [`is_motor_ready()`].
///
/// # Example
///
/// ```no_run
/// # use kartoffel::*;
/// #
/// loop {
///     motor_wait();
///     motor_turn_left();
///     motor_wait();
///     motor_step();
/// }
/// ```
#[inline(always)]
pub fn motor_wait() {
    while !is_motor_ready() {
        //
    }
}

/// Moves the bot one tile forward in the direction it's facing.
///
/// # Cooldown
///
/// ```text
/// 20_000 +- 15% ticks (~310 ms)
/// ```
///
/// # Example
///
/// ```no_run
/// # use kartoffel::*;
/// #
/// motor_wait();
/// motor_step();
/// ```
#[inline(always)]
pub fn motor_step() {
    wri(MEM_MOTOR, 0, 1);
}

/// Turns the bot left (counterclockwise).
///
/// # Cooldown
///
/// ```text
/// 10_000 +- 15% ticks (~150 ms)
/// ```
///
/// # Example
///
/// ```no_run
/// # use kartoffel::*;
/// #
/// motor_wait();
/// motor_turn_left();
/// ```
pub fn motor_turn_left() {
    motor_turn(-1);
}

/// Turns the bot right (clockwise).
///
/// # Cooldown
///
/// ```text
/// 10_000 +- 15% ticks (~150 ms)
/// ```
///
/// # Example
///
/// ```no_run
/// # use kartoffel::*;
/// #
/// motor_wait();
/// motor_turn_right();
/// ```
pub fn motor_turn_right() {
    motor_turn(1);
}

/// Turns the bot depending on the parameter:
///
/// - if dir < 0, the bot turns left (counterclockwise),
/// - if dir > 0, the bot turns right (clockwise),
/// - if dir = 0, the bot does nothing.
///
/// Only the sign of `dir` matters, `motor_turn(-123)` is the same as
/// `motor_turn(-1)`.
///
/// # Cooldown
///
/// ```text
/// 10_000 +- 15% ticks (~150 ms)
/// ```
///
/// # Example
///
/// ```no_run
/// # use kartoffel::*;
/// #
/// motor_wait();
/// motor_turn(-1);
///
/// motor_wait();
/// motor_turn(1);
/// ```
#[inline(always)]
pub fn motor_turn(dir: i32) {
    wri(MEM_MOTOR, 1, dir as u32);
}

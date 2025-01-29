use crate::{cmd, rdi, wri, MEM_MOTOR};

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
    wri(MEM_MOTOR, 0, cmd(0x01, 0x00, 0x00, 0x00));
}

/// Turns the bot.
///
/// Note that this is a low-level function - for convenience you'll most likely
/// want to use [`motor_turn_left()`] or [`motor_turn_right()`].
///
/// # Input
///
/// - if dir == -1, the bot turns left (counterclockwise),
/// - if dir == 1, the bot turns right (clockwise),
/// - if dir == 0, the bot does nothing.
///
/// Other values of `dir` are illegal and will crash the firmware.
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
/// motor_turn(-1); // turns left (counterclockwise)
///
/// motor_wait();
/// motor_turn(1); // turns right (clockwise)
/// ```
#[inline(always)]
pub fn motor_turn(dir: i8) {
    wri(MEM_MOTOR, 0, cmd(0x02, dir as u8, 0x00, 0x00));
}

/// Turns the bot left (counterclockwise).
///
/// See also: [`motor_turn()`], [`motor_turn_right()`].
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
#[inline(always)]
pub fn motor_turn_left() {
    motor_turn(-1);
}

/// Turns the bot right (clockwise).
///
/// See also: [`motor_turn()`], [`motor_turn_left()`].
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
#[inline(always)]
pub fn motor_turn_right() {
    motor_turn(1);
}

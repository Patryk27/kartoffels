use crate::{cmd, rdi, wri, MEM_MOTOR};

/// Returns whether the motor is ready and [`motor_pulse()`] can be invoked.
///
/// See also: [`motor_wait()`].
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
///     motor_step_fw();
/// }
/// ```
#[inline(always)]
pub fn motor_wait() {
    while !is_motor_ready() {
        //
    }
}

/// Sends a pulse to the motors.
///
/// Outcome depends on the parameters - legal combinations are:
///
/// - `(1, 1)` - bot drives forward,
/// - `(-1, -1)` - bot drives backward,
/// - `(-1, 1)` - bot turns left (counterclockwise),
/// - `(1, -1)` - bot turns right (clockwise).
///
/// Other values will cause the CPU to crash.
///
/// Note that this is a low-level function - for convenience you'll most likely
/// want to use one of:
///
/// - [`motor_step_fw()`],
/// - [`motor_step_bw()`],
/// - [`motor_turn_left()`],
/// - [`motor_turn_right()`].
///
/// # Cooldown
///
/// Depends on the parameters, see:
///
/// - [`motor_step_fw()`],
/// - [`motor_step_bw()`],
/// - [`motor_turn_left()`],
/// - [`motor_turn_right()`],
#[inline(always)]
pub fn motor_pulse(left: i8, right: i8) {
    wri(MEM_MOTOR, 0, cmd(0x01, left as u8, right as u8, 0x00));
}

/// Moves the bot one tile forward in the direction it's facing.
///
/// See also: [`motor_step_bw()`], [`motor_pulse()`].
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
/// motor_step_fw();
/// ```
#[inline(always)]
pub fn motor_step_fw() {
    motor_pulse(1, 1);
}

/// Moves the bot one tile away (backward) from the direction it's facing.
///
/// See also: [`motor_step_fw()`], [`motor_pulse()`].
///
/// # Cooldown
///
/// ```text
/// 30_000 +- 15% ticks (~468 ms)
/// ```
///
/// # Example
///
/// ```no_run
/// # use kartoffel::*;
/// #
/// motor_wait();
/// motor_step_bw();
/// ```
#[inline(always)]
pub fn motor_step_bw() {
    motor_pulse(-1, -1);
}

/// Turns the bot to its left (i.e. counterclockwise).
///
/// See also: [`motor_turn_right()`], [`motor_pulse()`].
///
/// # Cooldown
///
/// ```text
/// 25_000 +- 15% ticks (~390 ms)
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
    motor_pulse(-1, 1);
}

/// Turns the bot to its right (i.e. clockwise).
///
/// See also: [`motor_turn_left()`], [`motor_pulse()`].
///
/// # Cooldown
///
/// ```text
/// 25_000 +- 15% ticks (~390 ms)
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
    motor_pulse(1, -1);
}

use crate::*;

#[doc(hidden)]
pub const MOTOR_MEM: u32 = MEM + 3 * 1024;

/// Status returned when command succeeded.
///
/// See: [`IRQ_MOTOR_BUSY`].
pub const MOTOR_STAT_OK: u8 = 0x01;

/// Status returned when command failed.
///
/// See: [`IRQ_MOTOR_BUSY`].
pub const MOTOR_STAT_ERR: u8 = 0xff;

/// Error returned when you try to drive, but you get blocked by something.
///
/// See: [`IRQ_MOTOR_BUSY`], [`motor_pulse()`].
pub const MOTOR_ERR_BLOCKED: u8 = 0x01;

/// Interrupt raised when motor becomes busy.
///
/// Note that this interrupt is raised only when motor _becomes_ busy - if
/// motor was already busy when you ran a command, the interrupt will not be
/// raised.
///
/// See: [`irq_set()`], [`IRQ_MOTOR_IDLE`].
///
/// # Example
///
/// ```no_run
/// # use kartoffel::{*, print, println};
/// #
/// fn on_motor_busy(args: u32) {
///     let [_, dir, status, looking_at] = args.to_le_bytes();
///
///     if status == MOTOR_STAT_OK {
///         match dir {
///             b'^' => println!("stepped forward"),
///             b'>' => println!("turned right"),
///             b'v' => println!("stepped backward"),
///             b'<' => println!("turned left"),
///             _ => unreachable!(),
///         }
///
///         println!("looking at `{}`", looking_at as char);
///     } else {
///         match dir {
///             b'^' => println!("couldn't step forward"),
///             b'>' => println!("couldn't turn right"),
///             b'v' => println!("couldn't step backward"),
///             b'<' => println!("couldn't turn left"),
///             _ => unreachable!(),
///         }
///     }
///
///     println!();
/// }
///
/// irq_set(IRQ_MOTOR_BUSY, irq!(on_motor_busy));
///
/// motor_turn_left();
/// motor_wait();
/// motor_turn_right();
/// motor_wait();
/// motor_step_fw();
///
/// loop {}
/// ```
pub const IRQ_MOTOR_BUSY: u8 = 3;

/// Interrupt raised when motor becomes idle.
///
/// See: [`irq_set()`], [`IRQ_MOTOR_BUSY`].
///
/// # Example
///
/// ```no_run
/// # use kartoffel::{*, println};
/// #
/// fn on_motor_busy() {
///     println!("motor busy");
/// }
///
/// fn on_motor_idle() {
///     println!("motor idle");
/// }
///
/// irq_set(IRQ_MOTOR_BUSY, irq!(on_motor_busy));
/// irq_set(IRQ_MOTOR_IDLE, irq!(on_motor_idle));
///
/// motor_turn_left();
///
/// loop {}
/// ```
pub const IRQ_MOTOR_IDLE: u8 = 4;

/// Returns whether motor is ready.
///
/// See: [`motor_wait()`], [`IRQ_MOTOR_BUSY`], [`IRQ_MOTOR_IDLE`].
///
/// # Example
///
/// ```no_run
/// # use kartoffel::*;
/// #
/// if motor_ready() {
///     motor_step_fw();
/// }
/// ```
pub fn motor_ready() -> bool {
    unsafe { rdi(MOTOR_MEM, 0) == 1 }
}

/// Waits until motor is ready.
///
/// See: [`motor_ready()`], [`IRQ_MOTOR_BUSY`], [`IRQ_MOTOR_IDLE`].
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
pub fn motor_wait() {
    while !motor_ready() {
        //
    }
}

/// Sends a pulse to motor.
///
/// Outcome depends on the parameters - legal combinations are:
///
/// - `(1, 1)` - bot drives forward,
/// - `(-1, -1)` - bot drives backward,
/// - `(-1, 1)` - bot turns left (counterclockwise),
/// - `(1, -1)` - bot turns right (clockwise).
///
/// Other values will cause the firmware to crash.
///
/// This is a low-level function - for convenience you'll most likely want to
/// use one of:
///
/// - [`motor_step_fw()`],
/// - [`motor_step_bw()`],
/// - [`motor_turn_left()`],
/// - [`motor_turn_right()`].
///
/// # Cooldown
///
/// Depends on the pulse, see:
///
/// - [`motor_step_fw()`],
/// - [`motor_step_bw()`],
/// - [`motor_turn_left()`],
/// - [`motor_turn_right()`],
///
/// # Interrupts
///
/// - [`IRQ_MOTOR_BUSY`],
/// - [`IRQ_MOTOR_IDLE`].
pub fn motor_pulse(left: i8, right: i8) {
    unsafe {
        wri(MOTOR_MEM, 0, pack(0x01, left as u8, right as u8, 0x00));
    }
}

/// Moves the bot one tile forward in the direction it's facing.
///
/// See: [`motor_step_bw()`], [`motor_pulse()`].
///
/// # Cooldown
///
/// ~20k ticks (~310 ms)
///
/// # Interrupts
///
/// - [`IRQ_MOTOR_BUSY`],
/// - [`IRQ_MOTOR_IDLE`].
///
/// # Example
///
/// ```no_run
/// # use kartoffel::*;
/// #
/// motor_wait();
/// motor_step_fw();
/// ```
pub fn motor_step_fw() {
    motor_pulse(1, 1);
}

/// Moves the bot one tile away (backward) from the direction it's facing.
///
/// See: [`motor_step_fw()`], [`motor_pulse()`].
///
/// # Cooldown
///
/// ~30k ticks (~468 ms)
///
/// # Interrupts
///
/// - [`IRQ_MOTOR_BUSY`],
/// - [`IRQ_MOTOR_IDLE`].
///
/// # Example
///
/// ```no_run
/// # use kartoffel::*;
/// #
/// motor_wait();
/// motor_step_bw();
/// ```
pub fn motor_step_bw() {
    motor_pulse(-1, -1);
}

/// Turns the bot to its left (i.e. counterclockwise).
///
/// See: [`motor_turn_right()`], [`motor_pulse()`].
///
/// # Cooldown
///
/// ~25k ticks (~390 ms)
///
/// # Interrupts
///
/// - [`IRQ_MOTOR_BUSY`],
/// - [`IRQ_MOTOR_IDLE`].
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
    motor_pulse(-1, 1);
}

/// Turns the bot to its right (i.e. clockwise).
///
/// See: [`motor_turn_left()`], [`motor_pulse()`].
///
/// # Cooldown
///
/// ~25k ticks (~390 ms)
///
/// # Interrupts
///
/// - [`IRQ_MOTOR_BUSY`],
/// - [`IRQ_MOTOR_IDLE`].
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
    motor_pulse(1, -1);
}

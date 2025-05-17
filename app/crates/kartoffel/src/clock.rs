use crate::*;

#[doc(hidden)]
pub const CLOCK_MEM: u32 = MEM;

/// Index of the first timer.
///
/// See: [`timer_set()`].
///
/// # Example
///
/// ```no_run
/// # use kartoffel::*;
/// #
/// // Configure timer 0 to fire every 4*32=128 ticks
/// timer_set(TIMER0, TIMER_PS_4, 32);
/// ```
pub const TIMER0: u8 = 0;

/// Index of the second timer.
///
/// See: [`timer_set()`].
///
/// # Example
///
/// ```no_run
/// # use kartoffel::*;
/// #
/// // Configure timer 1 to fire every 4*32=128 ticks
/// timer_set(TIMER1, TIMER_PS_4, 32);
/// ```
pub const TIMER1: u8 = 1;

/// Index of the third timer.
///
/// See: [`timer_set()`].
///
/// # Example
///
/// ```no_run
/// # use kartoffel::*;
/// #
/// // Configure timer 2 to fire every 4*32=128 ticks
/// timer_set(TIMER2, TIMER_PS_4, 32);
/// ```
pub const TIMER2: u8 = 2;

/// Stops timer.
///
/// See: [`timer_set()`].
///
/// # Example
///
/// ```no_run
/// # use kartoffel::*;
/// #
/// // Stop timer 0
/// timer_set(0, TIMER_OFF, 0);
/// ```
pub const TIMER_OFF: u8 = 0;

/// Sets timer's prescaler to 4, i.e. timer clock = cpu clock / 4.
///
/// See: [`timer_set()`].
///
/// # Example
///
/// ```no_run
/// # use kartoffel::*;
/// #
/// // Configure timer to fire every 4*128=512 ticks (125 times a second)
/// timer_set(TIMER0, TIMER_PS_4, 128);
/// ```
pub const TIMER_PS_4: u8 = 1;

/// Sets timer's prescaler to 8, i.e. timer clock = cpu clock / 8.
///
/// See: [`timer_set()`].
///
/// # Example
///
/// ```no_run
/// # use kartoffel::*;
/// #
/// // Configure timer to fire every 8*128=1024 ticks (62.5 times a second)
/// timer_set(TIMER0, TIMER_PS_8, 128);
///
/// // Configure timer to fire every 8*250=2000 ticks (32 times a second)
/// timer_set(TIMER0, TIMER_PS_8, 250);
/// ```
pub const TIMER_PS_8: u8 = 2;

/// Sets timer's prescaler to 16, i.e. timer clock = cpu clock / 16.
///
/// See: [`timer_set()`].
///
/// # Example
///
/// ```no_run
/// # use kartoffel::*;
/// #
/// // Configure timer to fire every 16*128=2048 ticks (31.25 times a second)
/// timer_set(TIMER0, TIMER_PS_16, 128);
///
/// // Configure timer to fire every 16*250=4000 ticks (16 times a second)
/// timer_set(TIMER0, TIMER_PS_16, 250);
/// ```
pub const TIMER_PS_16: u8 = 3;

/// Sets timer's prescaler to 32, i.e. timer clock = cpu clock / 32.
///
/// See: [`timer_set()`].
///
/// # Example
///
/// ```no_run
/// # use kartoffel::*;
/// #
/// // Configure timer to fire every 32*128=4096 ticks (15.625 times a second)
/// timer_set(TIMER0, TIMER_PS_32, 128);
///
/// // Configure timer to fire every 32*250=8000 ticks (8 times a second)
/// timer_set(TIMER0, TIMER_PS_32, 250);
/// ```
pub const TIMER_PS_32: u8 = 4;

/// Sets timer's prescaler to 64, i.e. timer clock = cpu clock / 64.
///
/// See: [`timer_set()`].
///
/// # Example
///
/// ```no_run
/// # use kartoffel::*;
/// #
/// // Configure timer to fire every 64*128=8192 ticks (7.8125 times a second)
/// timer_set(TIMER0, TIMER_PS_64, 128);
///
/// // Configure timer to fire every 64*250=16000 ticks (4 times a second)
/// timer_set(TIMER0, TIMER_PS_64, 250);
/// ```
pub const TIMER_PS_64: u8 = 5;

/// Sets timer's prescaler to 128, i.e. timer clock = cpu clock / 128.
///
/// See: [`timer_set()`].
///
/// # Example
///
/// ```no_run
/// # use kartoffel::*;
/// #
/// // Configure timer to fire every 128*128=16384 ticks (3.90625 times a second)
/// timer_set(TIMER0, TIMER_PS_128, 128);
///
/// // Configure timer to fire every 128*250=32000 ticks (2 times a second)
/// timer_set(TIMER0, TIMER_PS_128, 250);
/// ```
pub const TIMER_PS_128: u8 = 6;

/// Sets timer's prescaler to 256, i.e. timer clock = cpu clock / 256.
///
/// See: [`timer_set()`].
///
/// # Example
///
/// ```no_run
/// # use kartoffel::*;
/// #
/// // Configure timer to fire every 128*256=32768 ticks (1.953125 times a second)
/// timer_set(TIMER0, TIMER_PS_256, 128);
///
/// // Configure timer to fire every 250*256=64000 (every second)
/// timer_set(TIMER0, TIMER_PS_256, 250);
/// ```
pub const TIMER_PS_256: u8 = 7;

/// Configures timer as non-repeating.
///
/// See: [`timer_set()`].
///
/// # Example
///
/// ```no_run
/// # use kartoffel::{*, print};
/// #
/// fn on_timer0() {
///     print!("a ");
/// }
///
/// fn on_timer1() {
///     print!("b ");
/// }
///
/// irq_set(IRQ_TIMER0, irq!(on_timer0));
/// irq_set(IRQ_TIMER1, irq!(on_timer1));
///
/// // Prints "a b a a a a a ..."
/// timer_set(TIMER0, TIMER_PS_256, 250);
/// timer_set(TIMER1, TIMER_PS_256 | TIMER_ONESHOT, 125);
/// ```
pub const TIMER_ONESHOT: u8 = 8;

/// Interrupt raised when [`TIMER0`] fires.
///
/// See: [`timer_set()`].
///
/// See also: [`IRQ_TIMER1`], [`IRQ_TIMER2`], [`irq_set()`].
pub const IRQ_TIMER0: u8 = 0;

/// Interrupt raised when [`TIMER1`] fires.
///
/// See: [`timer_set()`].
///
/// See also: [`IRQ_TIMER0`], [`IRQ_TIMER2`], [`irq_set()`].
pub const IRQ_TIMER1: u8 = 1;

/// Interrupt raised when [`TIMER2`] fires.
///
/// See: [`timer_set()`].
///
/// See also: [`IRQ_TIMER0`], [`IRQ_TIMER1`], [`irq_set()`].
pub const IRQ_TIMER2: u8 = 2;

/// Returns a pseudorandom number that can be used as a source of randomness
/// for hashmaps and the like.
///
/// Note that this doesn't return a *new* random number each time it's called -
/// the number is randomized once, when the bot is born.
pub fn clock_seed() -> u32 {
    unsafe { rdi(CLOCK_MEM, 0) }
}

/// Returns the number of ticks that have passed since the bot's been born.
///
/// This function returns the lower 32-bits of the underlying 64-bit counter,
/// which means it overflows (starts counting from zero) after every ~18 hours.
pub fn clock_ticks() -> u32 {
    unsafe { rdi(CLOCK_MEM, 1) }
}

/// Waits until given number of ticks has passed.
///
/// # Example
///
/// ```no_run
/// # use kartoffel::*;
/// #
/// clock_wait(64000); // waits for one second
/// ```
pub fn clock_wait(ticks: u32) {
    let ticks = clock_ticks() + ticks;

    while clock_ticks() < ticks {
        //
    }
}

/// Configures a timer.
///
/// Timers are counters that work independently from your code - they count down
/// to zero, raise an interrupt, and then restart (or not, it's configurable).
///
/// What makes timers special is that you don't need to actively take care of
/// them - the CPU decrements each timer's counter automatically, which not only
/// allows you to measure time much more accurately than a manual loop ever
/// could, but it's also free (as in: it doesn't take any CPU cycles from you):
///
/// ```no_run
/// # use kartoffel::{*, print, println};
/// #
/// fn yay() {
///     println!("yay ");
/// }
///
/// // On the IRQ_TIMER0 interrupt, run yay()
/// irq_set(IRQ_TIMER0, irq!(yay));
///
/// // Configure timer 0 to raise the IRQ_TIMER0 interrupt every 256*250=64000
/// // ticks, i.e. every second
/// timer_set(TIMER0, TIMER_PS_256, 250);
///
/// loop {
///     // do whatever - drive, wait, calculate something, ...
/// }
/// ```
///
/// Each kartoffel has three independent timers: [`TIMER0`], [`TIMER1`], and
/// [`TIMER2`]; they all work in the same way.
///
/// See: [`irq_set()`].
///
/// # Example
///
/// ```no_run
/// # use kartoffel::{*, print, println};
/// #
/// fn on_timer0() {
///     println!("timer0 ");
/// }
///
/// fn on_timer1() {
///     print!("timer1 ");
/// }
///
/// irq_set(IRQ_TIMER0, irq!(on_timer0));
/// irq_set(IRQ_TIMER1, irq!(on_timer1));
///
/// timer_set(TIMER0, TIMER_PS_256, 250);
/// timer_set(TIMER1, TIMER_PS_256, 125);
/// ```
///
/// # One-shot timers
///
/// By default, timers are continuous - after counting down to zero, they start
/// from `max` again. You can change this behavior using [`TIMER_ONESHOT`]:
///
/// ```no_run
/// # use kartoffel::{*, print};
/// #
/// fn on_timer0() {
///     print!("a ");
/// }
///
/// fn on_timer1() {
///     print!("b ");
/// }
///
/// irq_set(IRQ_TIMER0, irq!(on_timer0));
/// irq_set(IRQ_TIMER1, irq!(on_timer1));
///
/// // Prints "a b a a a a a ..."
/// timer_set(TIMER0, TIMER_PS_256, 250);
/// timer_set(TIMER1, TIMER_PS_256 | TIMER_ONESHOT, 125);
/// ```
///
/// # Using timers without interrupts
///
/// Instead of using interrupts, you can monitor a timer by hand - this makes it
/// easy to create watchdog mechanisms with explicit interrupt points:
///
/// ```no_run
/// # use kartoffel::{*, println};
/// #
/// fn heavy_algorithm() -> Result<(), &'static str> {
///     timer_set(TIMER0, TIMER_PS_256 | TIMER_ONESHOT, 3 * 250);
///
///     for x in 2..32 {
///         for y in 2..32 {
///             if !timer_on(TIMER0) {
///                 /* do some clean-up, whatever */
///                 return Err("ran out of time");
///             }
///
///             for z in 2..32 {
///                 if x * x + y * y == z * z {
///                     println!("{x},{y},{z}");
///                 }
///             }
///         }
///     }
///
///     Ok(())
/// }
///
/// match heavy_algorithm() {
///     Ok(_) => println!("ok"),
///     Err(err) => println!("err: {err}"),
/// }
/// ```
pub fn timer_set(idx: u8, cfg: u8, max: u16) {
    timer_set_ex(idx, cfg, 0, max);
}

/// Configures a timer (low-level variant).
///
/// For the most part, this function is the same as [`timer_set()`] - the extra
/// trick is that it allows you to specify accumulator's initial value:
///
/// ```no_run
/// # use kartoffel::{*, println};
/// #
/// fn yay() {
///     println!("yay ");
/// }
///
/// irq_set(IRQ_TIMER0, irq!(yay));
///
/// // Configure timer 0 to count from 125 down down to 0, and then restart,
/// // counting from 500 down to 0
/// timer_set_ex(TIMER0, TIMER_PS_256, 125, 500);
/// ```
///
/// When going through [`timer_set()`], the timer's accumulator is set to the
/// timer's maximum value - but sometimes it might be handy to have the
/// interrupt be raised earlier, in which case this function might come useful.
///
/// Note that specifying `acc = 0` is the same as saying `acc = max`, the
/// minimum value you should set `acc` to in order for the interrupt to be
/// raised earlier is `acc = 1`.
pub fn timer_set_ex(idx: u8, cfg: u8, acc: u8, max: u16) {
    if idx >= 3 {
        return;
    }

    let [max_lo, max_hi] = max.to_le_bytes();

    unsafe {
        wri(
            CLOCK_MEM,
            10 + 2 * idx as usize,
            pack(cfg, acc, max_lo, max_hi),
        );
    }
}

fn timer_lo(idx: u8) -> u32 {
    if idx >= 3 {
        return 0;
    }

    unsafe { rdi(CLOCK_MEM, 10 + 2 * idx as usize) }
}

fn timer_hi(idx: u8) -> u32 {
    if idx >= 3 {
        return 0;
    }

    unsafe { rdi(CLOCK_MEM, 11 + 2 * idx as usize) }
}

/// Returns whether timer is active or not.
///
/// # Example
///
/// ```no_run
/// # use kartoffel::{*, println};
/// #
/// fn heavy_algorithm() -> Result<(), &'static str> {
///     timer_set(TIMER0, TIMER_PS_256 | TIMER_ONESHOT, 3 * 250);
///
///     for x in 2..32 {
///         for y in 2..32 {
///             if !timer_on(TIMER0) {
///                 /* do some clean-up, whatever */
///                 return Err("ran out of time");
///             }
///
///             for z in 2..32 {
///                 if x * x + y * y == z * z {
///                     println!("{x},{y},{z}");
///                 }
///             }
///         }
///     }
///
///     Ok(())
/// }
///
/// match heavy_algorithm() {
///     Ok(_) => println!("ok"),
///     Err(err) => println!("err: {err}"),
/// }
/// ```
pub fn timer_on(idx: u8) -> bool {
    timer_cfg(idx) & 0b111 > 0
}

/// Returns timer's configuration.
pub fn timer_cfg(idx: u8) -> u8 {
    timer_lo(idx).to_le_bytes()[0]
}

/// Returns timer's accumulator value.
pub fn timer_acc(idx: u8) -> u16 {
    let [acc_lo, acc_hi, ..] = timer_hi(idx).to_le_bytes();

    u16::from_le_bytes([acc_lo, acc_hi])
}

/// Returns timer's maximum value.
pub fn timer_max(idx: u8) -> u16 {
    let [.., max_lo, max_hi] = timer_hi(idx).to_le_bytes();

    u16::from_le_bytes([max_lo, max_hi])
}

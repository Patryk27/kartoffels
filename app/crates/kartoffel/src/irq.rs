use crate::*;

#[doc(hidden)]
pub const IRQ_MEM: u32 = MEM + 1024;

/// Registers an interrupt handler.
///
/// Interrupts are like notifications - various peripherals emit interrupts and
/// [`irq_set()`] allows you to create a function that is called when a specific
/// interrupt is raised; it's a callback, essentially.
///
/// Interrupts are handy, because your code doesn't have to actively monitor
/// them - you just register a handler to be called when something occurs and
/// that's it, you don't have to worry about it anymore.
///
/// See also: [`irq_get()`], [`irq_take()`], [`irq_replace()`].
///
/// # Example
///
/// See: [`timer_set()`], [`IRQ_MOTOR_BUSY`], [`IRQ_COMPASS_READY`] etc.
///
/// # Multithreading
///
/// Interrupt handler - the function you pass here - is not called from another
/// thread, there's no multithreading involved. Rather, when a peripheral raises
/// an interrupt ("sends a notification"), the CPU simply jumps into the
/// interrupt handler.
///
/// What this means in practice is that your interrupt handlers should be
/// relatively small and bounded, e.g. this:
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
///
///     loop {}
/// }
///
/// irq_set(IRQ_TIMER0, irq!(on_timer0));
/// irq_set(IRQ_TIMER1, irq!(on_timer1));
///
/// timer_set(TIMER0, TIMER_PS_256, 250);
/// timer_set(TIMER1, TIMER_PS_256, 125);
/// ```
///
/// ... will print `timer0`, then `timer1`, and then enter an infinite loop,
/// since the `on_timer1()` function never returns.
///
/// Note that it is _not_ invalid or illegal for an interrupt handler to never
/// return (although it has limited utility).
///
/// # Nested interrupts
///
/// When an interrupt handler is working, other interrupts are noted down by the
/// CPU, but they are not executed. Once the current interrupt handler returns,
/// the next active interrupt with the lowest index gets executed.
///
/// So [`IRQ_TIMER0`] has priority over [`IRQ_TIMER1`], which has priority over
/// [`IRQ_TIMER1`], which has priority over [`IRQ_MOTOR_BUSY`] etc.
///
/// # Multiple handlers
///
/// Each interrupt can have at most one handler - calling [`irq_set()`] twice on
/// the same interrupt removes the previous handler.
///
/// See [`irq_replace()`] if you're interested in managing a chain of handlers.
///
/// # Arguments
///
/// Interrupt handlers can accept a single argument, a 32-bit unsigned number.
///
/// This number consists of 4 bytes, where the first byte determines index of
/// the currently-running IRQ, while the remaining 3 bytes contain IRQ-specific
/// payload.
///
/// This can be used to create generic IRQ handlers, where a single function is
/// responsible for handling multiple different interrupts:
///
/// ```no_run
/// # use kartoffel::{*, print, println};
/// #
/// fn on_any_timer(args: u32) {
///     let [irq, arg0, arg1, arg2] = args.to_le_bytes();
///
///     // irq - common for all interrupts, corresponds to the IRQ_SOMETHING
///     //       constant (e.g. IRQ_TIMER0)
///     //
///     // arg0/arg1/arg2 - interpretation depends on the specific interrupt
///     //                  (see e.g. IRQ_MOTOR_BUSY)
///
///     match irq {
///         IRQ_TIMER0 => println!("timer0"),
///         IRQ_TIMER1 => print!("timer1 "),
///         _ => unreachable!(), // unreachable in this specific example, since
///                              // we're irq_set()-tting only IRQ_TIMER0 and
///                              // IRQ_TIMER1
///     }
/// }
///
/// irq_set(IRQ_TIMER0, irq!(on_any_timer));
/// irq_set(IRQ_TIMER1, irq!(on_any_timer));
///
/// timer_set(TIMER0, TIMER_PS_256, 250);
/// timer_set(TIMER1, TIMER_PS_256, 125);
/// ```
///
/// # Panics
///
/// IRQ handlers can panic - this will cause the entire firmware to panic, same
/// way as calling `panic!()` in a non-IRQ function does.
pub fn irq_set(irq: u8, fun: IrqFn) {
    unsafe {
        wri(IRQ_MEM, irq_idx(irq), fun.0 as usize as u32);
    }
}

/// Returns an interrupt handler.
///
/// See: [`irq_set()`], [`irq_take()`].
///
/// # Example
///
/// ```no_run
/// # use kartoffel::{*, println};
/// #
/// fn on_timer() {
///     println!("timer");
/// }
///
/// irq_set(IRQ_TIMER0, irq!(on_timer));
///
/// let fun = irq_get(IRQ_TIMER0).unwrap();
///
/// unsafe {
///     fun.call(0);
///     fun.call(0);
///     fun.call(0);
/// }
/// ```
pub fn irq_get(irq: u8) -> Option<IrqFn> {
    unsafe { irq_fn(rdi(IRQ_MEM, irq_idx(irq))) }
}

/// Unregisters an interrupt handler.
///
/// See: [`irq_set()`], [`irq_take()`].
pub fn irq_clear(irq: u8) {
    irq_take(irq);
}

/// Unregisters an interrupt handler and returns it.
///
/// This function is essentially [`irq_get()`] followed by [`irq_clear()`], but
/// atomic.
///
/// See: [`irq_set()`], [`irq_replace()`].
pub fn irq_take(irq: u8) -> Option<IrqFn> {
    unsafe { irq_fn(swi(IRQ_MEM, irq_idx(irq), 0)) }
}

/// Replace an interrupt handler.
///
/// This function is essentially [`irq_get()`] followed by [`irq_set()`], but
/// atomic; it can be used to build irq-chains.
///
/// # Example
///
/// ```no_run
/// # use kartoffel::{*, print};
/// #
/// fn move_forward() {
///     static mut PREV: Option<IrqFn> = None;
///
///     fn on_motor_busy(arg: u32) {
///         unsafe {
///             if let Some(prev) = PREV {
///                 prev.call(arg);
///             }
///         }
///
///         print!("two");
///     }
///
///     // Save previous handler to `PREV`
///     unsafe {
///         PREV = irq_replace(IRQ_MOTOR_BUSY, irq!(on_motor_busy));
///     }
///
///     // Call a function that raises the interrupt
///     motor_step();
///
///     // Restore previous handler
///     unsafe {
///         irq_replace(IRQ_MOTOR_BUSY, PREV);
///     }
/// }
///
/// #[no_mangle]
/// fn main() {
///     fn on_motor_busy() {
///         print!("one ");
///     }
///
///     irq_set(IRQ_MOTOR_BUSY, irq!(on_motor_busy));
///
///     // Prints `one two`:
///     move_forward();
/// }
/// ```
pub fn irq_replace(irq: u8, fun: impl Into<Option<IrqFn>>) -> Option<IrqFn> {
    unsafe {
        irq_fn(swi(
            IRQ_MEM,
            irq_idx(irq),
            fun.into().map(|fun| fun.0 as usize as u32).unwrap_or(0),
        ))
    }
}

fn irq_idx(irq: u8) -> usize {
    1 + irq as usize
}

unsafe fn irq_fn(ptr: u32) -> Option<IrqFn> {
    if ptr == 0 {
        None
    } else {
        Some(mem::transmute::<usize, IrqFn>(ptr as usize))
    }
}

/// IRQ handler, i.e. a function executed when an IRQ is raised.
///
/// See: [`irq_set()`], [`irq_get()`], [`irq_take()`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct IrqFn(extern "C" fn(u32));

impl IrqFn {
    /// Creates an IRQ handler.
    ///
    /// # Safety
    ///
    /// `fun()` must be a trampoline function consisting of three instructions:
    ///
    /// ```text
    /// auipc ...
    /// jalr ...
    /// mret
    /// ```
    ///
    /// This is the case when you use the [`irq!()`] macro.
    #[doc(hidden)]
    pub unsafe fn new(fun: extern "C" fn(u32)) -> Self {
        Self(fun)
    }

    /// Calls the underlying handler-function.
    ///
    /// # Safety
    ///
    /// - Function returned here can assume it's running within an IRQ
    ///   environment, in particular with the interrupts turned off.
    ///
    /// - Function returned here can assume the argument passed to it matches
    ///   something an actual IRQ would provide.
    pub unsafe fn call(self, arg: u32) {
        let fun = {
            let addr = self.0 as *const u32;

            // We can't return `self.0`, because - following `irq!()` - this
            // points at the `handler()` function, which ends with `mret`,
            // yielding it (mostly) useless for our caller.
            //
            // Instead, we extract address of the `trampoline()` function and
            // that's what we return here.

            let auipc = ptr::read_volatile(addr);
            let hi = (auipc as i32 >> 12) << 12;

            let jalr = ptr::read_volatile(addr.offset(1));
            let lo = (jalr as i32) >> 20;

            mem::transmute::<*const u32, extern "C" fn(u32)>(
                addr.byte_offset((hi + lo) as isize),
            )
        };

        fun(arg);
    }
}

#[doc(hidden)]
pub trait IrqHandler<M> {
    fn call(self, args: u32);
}

#[doc(hidden)]
pub struct IrqHandlerA;

#[doc(hidden)]
pub struct IrqHandlerB;

impl<T> IrqHandler<IrqHandlerA> for T
where
    T: Fn(),
{
    fn call(self, _: u32) {
        (self)();
    }
}

impl<T> IrqHandler<IrqHandlerB> for T
where
    T: Fn(u32),
{
    fn call(self, args: u32) {
        (self)(args);
    }
}

/// Converts function into an IRQ handler.
///
/// See: [`irq_set()`].
#[macro_export]
macro_rules! irq {
    ($fun:ident) => {{
        #[inline(always)]
        fn call<M>(fun: impl IrqHandler<M>, args: u32) {
            fun.call(args);
        }

        // We have to call `$fun` through a trampoline, because Rust's ABI is
        // unstable, i.e. we don't have a guarantee that the `_arg` we receive
        // in `handler()` will land in the same register within `IrqHandler::call()`.
        //
        // Note that this does seem to be the case at the moment - that is, at
        // least for integers, `extern "Rust"` == `extern "C"` - but let's be on
        // the safe side just in case.
        extern "C" fn trampoline(args: u32) {
            call($fun, args);
        }

        #[unsafe(naked)]
        extern "C" fn handler(_args: u32) {
            unsafe {
                ::core::arch::naked_asm!(
                    "
                    call {0}
                    mret
                    ",
                    sym trampoline,
                );
            }
        }

        unsafe { $crate::IrqFn::new(handler) }
    }};
}

use crate::{rdi, MEM_TIMER};

/// Returns a pseudorandom number that can be used as a source of randomness
/// for hashmaps and the like.
///
/// Note that this doesn't return a *new* random number each time it's called -
/// rather the number is randomized once, when the bot is being (re)started.
#[inline(always)]
pub fn timer_seed() -> u32 {
    rdi(MEM_TIMER, 0)
}

/// Returns the number of ticks that have passed since the bot's been born.
#[inline(always)]
pub fn timer_ticks() -> u32 {
    rdi(MEM_TIMER, 1)
}

/// Waits for until given number of ticks has passed.
///
/// # Example
///
/// ```no_run
/// # use kartoffel::*;
/// #
/// timer_wait(64000); // waits for one second
/// ```
#[inline(always)]
pub fn timer_wait(ticks: u32) {
    let ticks = timer_ticks() + ticks;

    while timer_ticks() < ticks {
        //
    }
}

use crate::*;

pub const TIMER_PS_0: u16 = 1;

pub const TIMER_PS_8: u16 = 2;

pub const TIMER_PS_64: u16 = 3;

pub const TIMER_PS_256: u16 = 4;

pub const TIMER_PS_1024: u16 = 5;

/// Returns a pseudorandom number that can be used as a source of randomness
/// for hashmaps and the like.
///
/// Note that this doesn't return a *new* random number each time it's called -
/// the number is randomized once, when the bot is born.
pub fn timer_seed() -> u32 {
    unsafe { rdi(MEM_TIMER, 0) }
}

/// Returns the number of ticks that have passed since the bot's been born.
///
/// This counter overflows after about 18 hours, after which it will start
/// counting from zero.
pub fn timer_ticks() -> u32 {
    unsafe { rdi(MEM_TIMER, 1) }
}

/// Waits until given number of ticks has passed.
///
/// # Example
///
/// ```no_run
/// # use kartoffel::*;
/// #
/// timer_wait(64000); // waits for one second
/// ```
pub fn timer_wait(ticks: u32) {
    let ticks = timer_ticks() + ticks;

    while timer_ticks() < ticks {
        //
    }
}

pub fn timer_start(idx: u8, cfg: u8, count: u16) {
    wri(
        MEM_TIMER,
        1 + idx as usize,
        (cfg as u32) << 16 | (count as u32),
    );
}

pub fn timer_reset(idx: u8) {
    timer_start(idx, u8::MAX, 0);
}

pub fn timer_stop(idx: u8) {
    timer_start(idx, 0, 0);
}

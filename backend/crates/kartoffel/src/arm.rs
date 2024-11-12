use crate::{rdi, wri, MEM_ARM};

/// Returns whether the arm is ready and [`arm_stab()`] can be invoked.
///
/// See also: [`arm_wait()`].
///
/// # Example
///
/// ```no_run
/// # use kartoffel::*;
/// #
/// if is_arm_ready() {
///     arm_stab();
/// } else {
///     // run, forrest, run!
/// }
/// ```
#[inline(always)]
pub fn is_arm_ready() -> bool {
    rdi(MEM_ARM, 0) == 1
}

/// Waits for the arm to become ready.
///
/// See also: [`is_arm_ready()`].
///
/// # Example
///
/// ```no_run
/// # use kartoffel::*;
/// #
/// arm_wait();
/// arm_stab();
/// ```
#[inline(always)]
pub fn arm_wait() {
    while !is_arm_ready() {
        //
    }
}

/// Stabs the bot in front of you, killing it; if there's no bot there, nothing
/// happens.
///
/// # Cooldown
///
/// ```text
/// 60_000 +- 15% ticks (~930 ms)
/// ```
///
/// # Example
///
/// ```no_run
/// # use kartoffel::*;
/// #
/// arm_wait();
/// arm_stab();
/// ```
#[inline(always)]
pub fn arm_stab() {
    wri(MEM_ARM, 0, 1);
}

// TODO
#[inline(always)]
pub fn arm_pick() {
    wri(MEM_ARM, 0, 2);
}

// TODO
#[inline(always)]
pub fn arm_drop(nth: u8) {
    wri(MEM_ARM, 0, u32::from_be_bytes([0, 0, nth, 3]));
}

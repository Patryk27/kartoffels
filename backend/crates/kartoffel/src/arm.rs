use crate::{rdi, wri, MEM_ARM};

/// Returns whether the arm is ready and [`arm_stab()`] can be invoked.
#[inline(always)]
pub fn is_arm_ready() -> bool {
    rdi(MEM_ARM, 0) == 1
}

#[inline(always)]
pub fn arm_wait() {
    while !is_arm_ready() {
        //
    }
}

/// Stabs the bot in front of us (if any), killing it.
///
/// Note that this function has a cooldown period of 15000 ticks, see:
/// [`is_arm_ready()`].
#[inline(always)]
pub fn arm_stab() {
    wri(MEM_ARM, 0, 1)
}

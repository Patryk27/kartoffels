use crate::{cmd, rdi, wri, MEM_ARM};

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

/// Stabs the robot in front of you, killing it.
///
/// If there's no robot there, nothing happens (but the cooldown is still
/// applied).
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
    wri(MEM_ARM, 0, cmd(0x01, 0x00, 0x00, 0x00));
}

/// Picks the object in front of you and puts it into the inventory under the
/// zeroth index, shifting previously-picked objects into further indices.
///
/// (that is: if you had object at idx=0, picking a new object would shift this
/// already-picked object into idx=1, and then the newly-picked object would
/// appear at idx=0.)
///
/// If there's no object in front of you or you don't have any more space in the
/// inventory, nothing happens (but the cooldown is still applied).
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
/// arm_pick();
/// ```
#[inline(always)]
pub fn arm_pick() {
    wri(MEM_ARM, 0, cmd(0x02, 0x00, 0x00, 0x00));
}

/// Takes object from the inventory and drops it in front of you, shifting
/// following objects into their previous indices.
///
/// (that is: if you had objects at idx=0, idx=1 and idx=2, then dropping the
/// object at idx=1 would move object idx=2 back to idx=1.)
///
/// If you don't have object with given index or the object can't be placed in
/// front of you (e.g. because you're looking at a wall), nothing happens (but
/// the cooldown is still applied).
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
/// arm_pick();
///
/// arm_wait();
/// arm_drop(0); // drops whatever got picked before
/// ```
#[inline(always)]
pub fn arm_drop(idx: u8) {
    wri(MEM_ARM, 0, cmd(0x03, idx, 0x00, 0x00));
}

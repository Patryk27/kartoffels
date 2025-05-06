use crate::*;

#[doc(hidden)]
pub const ARM_MEM: u32 = MEM + 4 * 1024;

/// Index of the [`arm_stab()`] command.
///
/// See: [`IRQ_ARM_BUSY`].
pub const ARM_CMD_STAB: u8 = 0x01;

/// Index of the [`arm_pick()`] command.
///
/// See: [`IRQ_ARM_BUSY`].
pub const ARM_CMD_PICK: u8 = 0x02;

/// Index of the [`arm_drop()`] command.
///
/// See: [`IRQ_ARM_BUSY`].
pub const ARM_CMD_DROP: u8 = 0x03;

/// Status returned when command succeeded.
///
/// See: [`IRQ_ARM_BUSY`].
pub const ARM_STAT_OK: u8 = 0x01;

/// Status returned when command failed.
///
/// See: [`IRQ_ARM_BUSY`].
///
/// See also: [`ARM_ERR_NOBODY_THERE`], [`ARM_ERR_NOTHING_THERE`] etc.
pub const ARM_STAT_ERR: u8 = 0xff;

/// Error returned when you try to stab, but there's nobody in front of you to
/// stab.
///
/// See: [`IRQ_ARM_BUSY`], [`arm_stab()`].
pub const ARM_ERR_NOBODY_THERE: u8 = 0x01;

/// Error returned when you try to pick, but there's nothing in front of you to
/// pick.
///
/// See: [`IRQ_ARM_BUSY`], [`arm_pick()`].
pub const ARM_ERR_NOTHING_THERE: u8 = 0x02;

/// Error returned when you try to pick, but the inventory is already full.
///
/// See: [`IRQ_ARM_BUSY`], [`arm_pick()`].
pub const ARM_ERR_INVENTORY_FULL: u8 = 0x03;

/// Error returned when you try to drop, but there's no floor in front of you on
/// which an object could be dropped.
///
/// See: [`IRQ_ARM_BUSY`], [`arm_drop()`].
pub const ARM_ERR_NEEDS_FLOOR: u8 = 0x04;

/// Error returned when you try to drop, but there's no such object in the
/// inventory.
///
/// See: [`IRQ_ARM_BUSY`], [`arm_drop()`].
pub const ARM_ERR_NO_SUCH_OBJECT: u8 = 0x05;

/// Interrupt raised when arm becomes busy.
///
/// Note that this interrupt is raised only when arm _becomes_ busy - if arm was
/// already busy when you ran a command, the interrupt will not be raised.
///
/// See: [`irq_set()`], [`IRQ_ARM_IDLE`].
///
/// # Example
///
/// ```no_run
/// # use kartoffel::{*, println};
/// #
/// fn on_arm_busy(args: u32) {
///     let [_, arg0, arg1, arg2] = args.to_le_bytes();
///
///     match arg0 {
///         ARM_CMD_STAB => {
///             if arg1 == ARM_STAT_ERR {
///                 println!("not stabbed - error #{arg2}");
///             } else {
///                 println!("stabbed");
///             }
///         }
///
///         ARM_CMD_PICK => {
///             if arg1 == ARM_STAT_ERR {
///                 println!("not picked - error #{arg2}");
///             } else {
///                 println!("picked {} into slot {arg2}", arg1 as char);
///             }
///         }
///
///         ARM_CMD_DROP => {
///             if arg1 == ARM_STAT_ERR {
///                 println!("not dropped - error #{arg2}");
///             } else {
///                 println!("dropped {} from slot {arg2}", arg1 as char);
///             }
///         }
///
///         _ => unreachable!(),
///     }
/// }
///
/// irq_set(IRQ_ARM_BUSY, irq!(on_arm_busy));
/// arm_stab();
///
/// loop {}
/// ```
pub const IRQ_ARM_BUSY: u8 = 5;

/// Interrupt raised when arm becomes idle.
///
/// See: [`irq_set()`], [`IRQ_ARM_BUSY`].
///
/// # Example
///
/// ```no_run
/// # use kartoffel::{*, println};
/// #
/// fn on_arm_busy() {
///     println!("arm busy");
/// }
///
/// fn on_arm_idle() {
///     println!("arm idle");
/// }
///
/// irq_set(IRQ_ARM_BUSY, irq!(on_arm_busy));
/// irq_set(IRQ_ARM_IDLE, irq!(on_arm_idle));
///
/// arm_stab();
/// ```
pub const IRQ_ARM_IDLE: u8 = 6;

/// Returns whether arm is ready.
///
/// See: [`arm_wait()`], [`IRQ_ARM_BUSY`], [`IRQ_ARM_IDLE`].
///
/// # Example
///
/// ```no_run
/// # use kartoffel::*;
/// #
/// if arm_ready() {
///     arm_stab();
/// } else {
///     // run, forrest, run!
/// }
/// ```
pub fn arm_ready() -> bool {
    unsafe { rdi(ARM_MEM, 0) == 1 }
}

/// Waits until arm is ready.
///
/// See: [`arm_ready()`], [`IRQ_ARM_BUSY`], [`IRQ_ARM_IDLE`].
///
/// # Example
///
/// ```no_run
/// # use kartoffel::*;
/// #
/// arm_wait();
/// arm_stab();
/// ```
pub fn arm_wait() {
    while !arm_ready() {
        //
    }
}

/// Stabs the bot in front of you, killing it.
///
/// If there's nobody there, nothing happens (but the cooldown is still
/// applied).
///
/// # Cooldown
///
/// - ~60k ticks (~930 ms) on success,
/// - ~45k ticks (~703 ms) on failure (e.g. there's nobody to stab).
///
/// # Interrupts
///
/// - [`IRQ_ARM_BUSY`],
/// - [`IRQ_ARM_IDLE`].
///
/// # Example
///
/// ```no_run
/// # use kartoffel::*;
/// #
/// arm_wait();
/// arm_stab();
/// ```
pub fn arm_stab() {
    unsafe {
        wri(ARM_MEM, 0, pack(ARM_CMD_STAB, 0x00, 0x00, 0x00));
    }
}

/// Picks an object in front of you and puts it into the inventory.
///
/// Object is then put it into the inventory under the zeroth index, shifting
/// all previously-picked objects into further indices - i.e. if you had an
/// object at idx=0, picking a new object would shift this already-picked object
/// into idx=1, and then the newly-picked object would appear at idx=0.
///
/// If there's no object in front of you or you don't have any more space in the
/// inventory, nothing happens (but the cooldown is still applied).
///
/// # Cooldown
///
/// - ~60k ticks (~930 ms) on success,
/// - ~45k ticks (~703 ms) on failure (e.g. there's nothing to pick).
///
/// # Interrupts
///
/// - [`IRQ_ARM_BUSY`],
/// - [`IRQ_ARM_IDLE`].
///
/// # Example
///
/// ```no_run
/// # use kartoffel::*;
/// #
/// arm_wait();
/// arm_pick();
/// ```
pub fn arm_pick() {
    unsafe {
        wri(ARM_MEM, 0, pack(ARM_CMD_PICK, 0x00, 0x00, 0x00));
    }
}

/// Removes an object from the inventory and drops it in front of you.
///
/// Objects that remain in the inventory have their indices shifted back - e.g.
/// if you had objects at idx=0, idx=1 and idx=2, then dropping the object from
/// idx=1 would move object idx=2 at idx=1.
///
/// If you don't have object with given index or the object can't be placed in
/// front of you (e.g. because you're looking at a wall), nothing happens (but
/// the cooldown is still applied).
///
/// # Cooldown
///
/// - ~60k ticks (~930 ms) on success,
/// - ~45k ticks (~703 ms) on failure (e.g. inventory doesn't have object #idx).
///
/// # Interrupts
///
/// - [`IRQ_ARM_BUSY`],
/// - [`IRQ_ARM_IDLE`].
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
pub fn arm_drop(idx: u8) {
    unsafe {
        wri(ARM_MEM, 0, pack(ARM_CMD_DROP, idx, 0x00, 0x00));
    }
}

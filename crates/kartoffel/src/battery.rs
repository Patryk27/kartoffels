use crate::{rdi, MEM_BATTERY};

/// Returns the remaining battery energy.
///
/// Since battery is not simulated at the moment, this function doesn't come
/// useful.
#[doc(hidden)]
#[inline(always)]
pub fn battery_energy() -> u32 {
    rdi(MEM_BATTERY, 0)
}

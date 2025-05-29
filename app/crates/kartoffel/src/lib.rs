//! # kartoffel
//!
//! Building blocks for a kartoffel bot:
//!
//! ```no_run
//! #![no_std]
//! #![no_main]
//! #
//! # extern crate std;
//!
//! use kartoffel::*;
//!
//! #[unsafe(no_mangle)]
//! fn main() {
//!     loop {
//!         println!("small step for a bot!");
//!
//!         motor_wait();
//!         motor_step();
//!     }
//! }
//! ```
//!
//! ## Getting started
//!
//! - <https://github.com/patryk27/kartoffel>
//! - <https://github.com/patryk27/kartoffels>

#![no_std]

extern crate alloc;

mod allocator;
mod arm;
mod clock;
mod compass;
mod irq;
mod motor;
mod panic;
mod radar;
mod serial;

pub use self::arm::*;
pub use self::clock::*;
pub use self::compass::*;
pub use self::irq::*;
pub use self::motor::*;
pub use self::radar::*;
pub use self::serial::*;
use core::sync::atomic::{AtomicU32, Ordering};
use core::{fmt, mem, ptr};

const MEM: u32 = 0x08000000;

unsafe fn rdi(ptr: u32, off: usize) -> u32 {
    unsafe { ptr::read_volatile((ptr as *const u32).wrapping_add(off)) }
}

unsafe fn wri(ptr: u32, off: usize, val: u32) {
    unsafe {
        ptr::write_volatile((ptr as *mut u32).wrapping_add(off), val);
    }
}

unsafe fn swi(ptr: u32, off: usize, val: u32) -> u32 {
    unsafe {
        AtomicU32::from_ptr((ptr as *mut u32).wrapping_add(off))
            .swap(val, Ordering::SeqCst)
    }
}

#[doc(hidden)]
pub const fn pack(a: u8, b: u8, c: u8, d: u8) -> u32 {
    u32::from_le_bytes([a, b, c, d])
}

/// Interrupts the firmware.
///
/// Calling this function pauses the world and focuses interface on this bot,
/// which can be useful for debugging.
///
/// Note that this function doesn't work in online mode (it's a no-op there).
pub fn breakpoint() {
    #[cfg(target_arch = "riscv32")]
    unsafe {
        core::arch::asm!("ebreak");
    }
}

#[cfg(target_arch = "riscv32")]
core::arch::global_asm!(
    r#"
    .global _start
    .section .init, "ax"

    _start:
        la sp, _stack_end
        jal main
        j _end

    _end:
        j _end
    "#,
);

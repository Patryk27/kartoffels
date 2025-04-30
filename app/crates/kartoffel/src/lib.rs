//! # kartoffel
//!
//! This crate provides building blocks for implementing a kartoffel bot:
//!
//! - <https://github.com/patryk27/kartoffel>
//! - <https://github.com/patryk27/kartoffels>

#![no_std]

extern crate alloc;

mod allocator;
mod arm;
mod compass;
mod motor;
mod panic;
mod radar;
mod serial;
mod timer;

pub use self::arm::*;
pub use self::compass::*;
pub use self::motor::*;
pub use self::radar::*;
pub use self::serial::*;
pub use self::timer::*;
use core::ptr;

const MEM: u32 = 0x08000000;

/// Memory address for the clock peripheral.
///
/// Usually you don't have to use this directly, see [`timer_seed()`] and
/// similar functions.
pub const MEM_TIMER: u32 = MEM;

/// Memory address for the serial peripheral.
///
/// Usually you don't have to use this directly, see [`serial_write()`] and
/// similar functions.
pub const MEM_SERIAL: u32 = MEM + 2 * 1024;

/// Memory address for the motor peripheral.
///
/// Usually you don't have to use this directly, see [`motor_pulse()`] and
/// similar functions.
pub const MEM_MOTOR: u32 = MEM + 3 * 1024;

/// Memory address for the arm peripheral.
///
/// Usually you don't have to use this directly, see [`arm_stab()`] and similar
/// functions.
pub const MEM_ARM: u32 = MEM + 4 * 1024;

/// Memory address for the radar peripheral.
///
/// Usually you don't have to use this directly, see [`radar_scan()`] and
/// similar functions.
pub const MEM_RADAR: u32 = MEM + 5 * 1024;

/// Memory address for the compass peripheral.
///
/// Usually you don't have to use this directly, see [`compass_dir()`].
pub const MEM_COMPASS: u32 = MEM + 6 * 1024;

unsafe fn rdi(ptr: u32, off: usize) -> u32 {
    ptr::read_volatile((ptr as *const u32).wrapping_add(off))
}

unsafe fn wri(ptr: u32, off: usize, val: u32) {
    ptr::write_volatile((ptr as *mut u32).wrapping_add(off), val);
}

#[doc(hidden)]
pub const fn cmd(cmd: u8, arg0: u8, arg1: u8, arg2: u8) -> u32 {
    u32::from_le_bytes([cmd, arg0, arg1, arg2])
}

#[cfg(target_arch = "riscv32")]
core::arch::global_asm!(
    r#"
    .global _start
    .section .init, "ax"

    _start:
        la sp, _stack_end
        jal main
        ebreak
    "#,
);

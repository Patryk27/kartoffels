//! # kartoffel
//!
//! This crate provides basic building blocks for implementing custom robots for
//! the [kartoffels](https://kartoffels.pwy.io) game.
//!
//! For example usage, see <https://github.com/patryk27/kartoffel>.

#![no_std]

extern crate alloc;

mod allocator;
mod arm;
mod battery;
mod compass;
mod motor;
mod panic;
mod radar;
mod serial;
mod timer;

pub use self::arm::*;
pub use self::battery::*;
pub use self::compass::*;
pub use self::motor::*;
pub use self::radar::*;
pub use self::serial::*;
pub use self::timer::*;
use core::ptr;

const MEM: *mut u32 = 0x08000000 as *mut u32;
const MEM_TIMER: *mut u32 = MEM;
const MEM_BATTERY: *mut u32 = MEM.wrapping_byte_add(1024);
const MEM_SERIAL: *mut u32 = MEM.wrapping_byte_add(2 * 1024);
const MEM_MOTOR: *mut u32 = MEM.wrapping_byte_add(3 * 1024);
const MEM_ARM: *mut u32 = MEM.wrapping_byte_add(4 * 1024);
const MEM_RADAR: *mut u32 = MEM.wrapping_byte_add(5 * 1024);
const MEM_COMPASS: *mut u32 = MEM.wrapping_byte_add(6 * 1024);

#[inline(always)]
fn rdi(ptr: *mut u32, off: usize) -> u32 {
    unsafe { ptr::read_volatile(ptr.wrapping_add(off)) }
}

#[inline(always)]
fn wri(ptr: *mut u32, off: usize, val: u32) {
    unsafe {
        ptr::write_volatile(ptr.wrapping_add(off), val);
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
        ebreak
    "#,
);

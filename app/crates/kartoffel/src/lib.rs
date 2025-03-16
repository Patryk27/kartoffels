//! # kartoffel
//!
//! This crate provides building blocks for implementing a bot for the
//! [kartoffels](https://kartoffels.pwy.io) game - see:
//!
//! <https://github.com/patryk27/kartoffel>

#![no_std]

extern crate alloc;

mod allocator;
mod arm;
mod battery;
mod bluetooth;
mod compass;
mod motor;
mod panic;
mod radar;
mod serial;
mod timer;

pub use self::arm::*;
pub use self::battery::*;
pub use self::bluetooth::*;
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
const MEM_BLUETOOTH: *mut u32 = MEM.wrapping_byte_add(7 * 1024);

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

#[inline(always)]
fn cmd(cmd: u8, arg0: u8, arg1: u8, arg2: u8) -> u32 {
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

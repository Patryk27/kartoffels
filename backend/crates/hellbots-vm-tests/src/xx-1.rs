#![no_std]
#![cfg_attr(target_arch = "riscv64", no_main)]

use core::hint::black_box;

hellbots_pac::init!(+panic);

fn main() -> u32 {
    a() + b() * c()
}

#[inline(never)]
fn a() -> u32 {
    black_box(10)
}

#[inline(never)]
fn b() -> u32 {
    black_box(20)
}

#[inline(never)]
fn c() -> u32 {
    black_box(30)
}

/*
 * x1 = 1048588
 * x2 = 1179648
 * x10 = 610
 */

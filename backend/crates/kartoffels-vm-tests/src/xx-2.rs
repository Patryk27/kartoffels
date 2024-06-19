#![no_std]
#![cfg_attr(target_arch = "riscv64", no_main)]

extern crate alloc;

use alloc::vec::Vec;
use core::hint::black_box;

kartoffels_pac::init!(+panic, +alloc(4096));

fn main() -> u32 {
    let mut items = Vec::new();

    items.push(12345);

    for n in 0..32 {
        black_box(&mut items).push(n * n);
    }

    black_box(&items).iter().map(|item| item + 123).sum()
}

/*
 * x1 = 1048588
 * x2 = 1179648
 * x6 = 1
 * x7 = 1059728
 * x10 = 26820
 * x11 = 3048
 * x12 = 18014398509481984
 * x13 = 1060368
 * x14 = 1059736
 * x16 = 1
 * x17 = 1
 * x28 = 63
 * x29 = 127
 */

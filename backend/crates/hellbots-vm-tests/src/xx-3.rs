#![no_std]
#![cfg_attr(target_arch = "riscv64", no_main)]

extern crate alloc;

use alloc::collections::BTreeMap;
use core::hint::black_box;

hellbots_pac::init!(+panic, +alloc(4096));

fn main() -> u32 {
    let mut items = BTreeMap::<i128, i128>::new();

    for n in 0..32 {
        let n = black_box(n);

        black_box(&mut items).insert(n, n * n);
    }

    items
        .into_iter()
        .filter_map(|(k, v)| if k % 3 == 0 { Some(k + v) } else { None })
        .sum::<i128>() as u32
}

/*
 * x1 = 1051736
 * x2 = 1179648
 * x6 = 1060364
 * x7 = 1063080
 * x10 = 3630
 * x11 = 1179368
 * x13 = 1063736
 * x14 = 1063088
 * x16 = 1
 * x17 = -6148914691236517226
 * x28 = 63
 * x29 = 127
 */

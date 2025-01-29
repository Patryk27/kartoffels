#![cfg_attr(target_arch = "riscv32", no_std, no_main)]

extern crate alloc;
extern crate kartoffel;

use alloc::collections::BTreeMap;
use core::hint::black_box;

#[cfg_attr(target_arch = "riscv32", no_mangle)]
fn main() {
    let mut items = BTreeMap::<i128, i128>::new();

    for n in 0..32 {
        let n = black_box(n);

        black_box(&mut items).insert(n, n * n);
    }

    let out = items
        .into_iter()
        .filter_map(|(k, v)| if k % 3 == 0 { Some(k + v) } else { None })
        .sum::<i128>() as u32;

    kartoffels_cpu_tests::exit(out);
}

/*
 * x10 = 3630
 */

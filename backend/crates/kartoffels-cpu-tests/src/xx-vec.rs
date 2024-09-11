#![cfg_attr(target_arch = "riscv64", no_std, no_main)]

extern crate alloc;
extern crate kartoffel;

use alloc::vec::Vec;
use core::hint::black_box;

#[cfg_attr(target_arch = "riscv64", no_mangle)]
fn main() {
    let mut items = Vec::new();

    items.push(12345);

    for n in 0..32 {
        black_box(&mut items).push(n * n);
    }

    let out = black_box(&items).iter().map(|item| item + 123).sum();

    kartoffels_cpu_tests::exit(out);
}

/*
 * x10 = 26820
 */

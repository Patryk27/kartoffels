#![cfg_attr(target_arch = "riscv32", no_std, no_main)]

extern crate kartoffel;

use core::hint::black_box;

#[cfg_attr(target_arch = "riscv32", no_mangle)]
fn main() {
    let mut out = 0.0f32;

    for x in 0..128 {
        for y in 64..256 {
            out = black_box(out);
            out += x as f32;

            out = black_box(out);
            out *= y as f32;
        }
    }

    kartoffels_cpu_tests::exit(out.to_bits());
}

/*
 * x10 = 2139095040
 */

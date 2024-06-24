#![cfg_attr(target_arch = "riscv64", no_std, no_main)]

extern crate alloc;

#[cfg(target_arch = "riscv64")]
mod test {
    use alloc::vec::Vec;
    use core::hint::black_box;

    kartoffels_pac::init!(+panic, +alloc(4096));

    pub fn main() -> u32 {
        let mut items = Vec::new();

        items.push(12345);

        for n in 0..32 {
            black_box(&mut items).push(n * n);
        }

        black_box(&items).iter().map(|item| item + 123).sum()
    }
}

#[cfg(target_arch = "riscv64")]
fn main() -> u32 {
    test::main()
}

#[cfg(not(target_arch = "riscv64"))]
fn main() {
    //
}

/*
 * x1 = 1048588
 * x2 = 1179648
 * x6 = 1
 * x7 = 1059744
 * x10 = 26820
 * x11 = 3048
 * x12 = 18014398509481984
 * x13 = 1060384
 * x14 = 1059752
 * x16 = 1
 * x17 = 1
 * x28 = 63
 * x29 = 127
 */

#![cfg_attr(target_arch = "riscv64", no_std, no_main)]

extern crate alloc;

#[cfg(target_arch = "riscv64")]
mod test {
    use alloc::collections::BTreeMap;
    use core::hint::black_box;

    kartoffels_pac::init!(+panic, +alloc(4096));

    pub fn main() -> u32 {
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
 * x1 = 1051736
 * x2 = 1179648
 * x6 = 1060600
 * x7 = 1063096
 * x10 = 3630
 * x11 = 1179368
 * x13 = 1063752
 * x14 = 1063104
 * x16 = 1
 * x17 = -6148914691236517226
 * x28 = 63
 * x29 = 127
 */

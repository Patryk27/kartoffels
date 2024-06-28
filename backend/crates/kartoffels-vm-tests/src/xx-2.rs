#![cfg_attr(target_arch = "riscv64", no_std, no_main)]

extern crate alloc;
extern crate kartoffel;

#[cfg(target_arch = "riscv64")]
mod test {
    use alloc::vec::Vec;
    use core::hint::black_box;

    #[no_mangle]
    fn main() -> u32 {
        let mut items = Vec::new();

        items.push(12345);

        for n in 0..32 {
            black_box(&mut items).push(n * n);
        }

        black_box(&items).iter().map(|item| item + 123).sum()
    }
}

#[cfg(not(target_arch = "riscv64"))]
fn main() {
    //
}

/*
 * x10 = 26820
 */

#![cfg_attr(target_arch = "riscv64", no_std, no_main)]

extern crate kartoffel;

#[cfg(target_arch = "riscv64")]
mod test {
    use core::hint::black_box;

    #[no_mangle]
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
}

#[cfg(not(target_arch = "riscv64"))]
fn main() {
    //
}

/*
 * x10 = 610
 */

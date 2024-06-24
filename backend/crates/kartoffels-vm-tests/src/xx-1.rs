#![cfg_attr(target_arch = "riscv64", no_std, no_main)]

#[cfg(target_arch = "riscv64")]
mod test {
    use core::hint::black_box;

    kartoffels_pac::init!(+panic);

    pub fn main() -> u32 {
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
 * x10 = 610
 */

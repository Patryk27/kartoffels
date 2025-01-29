#![cfg_attr(target_arch = "riscv32", no_std)]

#[cfg(target_arch = "riscv32")]
#[macro_export]
macro_rules! test {
    ($code:literal) => {
        core::arch::global_asm!($code);

        #[panic_handler]
        fn panic(_info: &core::panic::PanicInfo) -> ! {
            loop {}
        }
    };
}

#[cfg(not(target_arch = "riscv32"))]
#[macro_export]
macro_rules! test {
    ($code:literal) => {
        fn main() {
            //
        }
    };
}

#[cfg(target_arch = "riscv32")]
pub fn exit(val: u32) {
    use core::arch::asm;

    unsafe {
        asm!("mv x10, {}", in(reg) val);
        asm!("ebreak");
    }
}

#[cfg(not(target_arch = "riscv32"))]
pub fn exit(val: u32) {
    println!("{val}");
}

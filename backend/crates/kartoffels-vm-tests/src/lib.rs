#![no_std]

#[cfg(target_arch = "riscv64")]
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

#[cfg(not(target_arch = "riscv64"))]
#[macro_export]
macro_rules! test {
    ($code:literal) => {
        fn main() {
            //
        }
    };
}

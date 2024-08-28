use core::panic::PanicInfo;

#[allow(dead_code)]
#[allow(clippy::empty_loop)]
#[cfg_attr(target_arch = "riscv64", panic_handler)]
fn panic(_: &PanicInfo) -> ! {
    loop {
        //
    }
}

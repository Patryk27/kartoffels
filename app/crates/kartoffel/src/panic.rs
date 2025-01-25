use crate::Serial;
use core::fmt::Write;
use core::panic::PanicInfo;

#[allow(dead_code)]
#[cfg_attr(target_arch = "riscv64", panic_handler)]
fn panic(info: &PanicInfo) -> ! {
    #[cfg(feature = "serial-panic")]
    let _ = write!(&mut Serial, "\n{info}");

    #[allow(clippy::empty_loop)]
    loop {
        //
    }
}

use crate::Serial;
use core::fmt::Write;
use core::panic::PanicInfo;

#[allow(dead_code)]
#[allow(clippy::empty_loop)]
#[cfg_attr(target_arch = "riscv64", panic_handler)]
fn panic(i: &PanicInfo) -> ! {
    let mut output = Serial;

    // Only print the panic message if the `serial-panic` feature is enabled.
    // Allows it to be disabled for smaller binaries.
    #[cfg(feature = "serial-panic")]
    let _ = write!(&mut output, "\n{}", i);

    loop {
        // Loop forever
    }
}

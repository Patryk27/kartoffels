use core::fmt::Write;
use core::panic::PanicInfo;

use crate::SerialOutput;

#[allow(dead_code)]
#[allow(clippy::empty_loop)]
#[cfg_attr(target_arch = "riscv64", panic_handler)]
fn panic(i: &PanicInfo) -> ! {
    let mut output = SerialOutput;

    let _ = write!(&mut output, "\n- KERNEL PANIC -\n");
    let _ = write!(&mut output, "{}", i);

    loop {
        // Loop forever
    }
}

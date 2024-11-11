#![cfg_attr(target_arch = "riscv64", no_std, no_main)]

use kartoffel::*;

#[cfg_attr(target_arch = "riscv64", no_mangle)]
fn main() {
    loop {
        motor_wait();
        motor_step();
    }
}

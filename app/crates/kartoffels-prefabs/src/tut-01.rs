//! Bot used for tutorial's tests.

#![cfg_attr(target_arch = "riscv32", no_std, no_main)]

use kartoffel::*;

#[cfg_attr(target_arch = "riscv32", no_mangle)]
fn main() {
    loop {
        motor_wait();
        motor_step_fw();

        motor_wait();
        motor_step_fw();

        motor_wait();
        motor_step_fw();

        motor_wait();
        motor_turn_right();
    }
}

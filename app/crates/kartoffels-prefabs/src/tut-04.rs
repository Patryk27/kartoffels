//! Bot used for tutorial's tests.

#![cfg_attr(target_arch = "riscv32", no_std, no_main)]

use kartoffel::*;

#[cfg_attr(target_arch = "riscv32", no_mangle)]
fn main() {
    loop {
        radar_wait();

        let scan = radar_scan_3x3();

        if scan.at(0, -1) == '@' {
            arm_wait();
            arm_stab();
        } else if scan.at(-1, 0) == '@' {
            motor_wait();
            motor_turn_left();
        } else if scan.at(1, 0) == '@' {
            motor_wait();
            motor_turn_right();
        } else {
            motor_wait();
            motor_step_fw();
        }
    }
}

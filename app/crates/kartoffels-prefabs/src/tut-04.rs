//! Bot used for tutorial's tests.

#![cfg_attr(target_arch = "riscv32", no_std, no_main)]

use kartoffel::*;

#[cfg_attr(target_arch = "riscv32", no_mangle)]
fn main() {
    loop {
        radar_wait();
        radar_scan(3);

        if radar_read(0, -1) == '@' {
            arm_wait();
            arm_stab();
        } else if radar_read(-1, 0) == '@' {
            motor_wait();
            motor_turn_left();
        } else if radar_read(1, 0) == '@' {
            motor_wait();
            motor_turn_right();
        } else {
            motor_wait();
            motor_step_fw();
        }
    }
}

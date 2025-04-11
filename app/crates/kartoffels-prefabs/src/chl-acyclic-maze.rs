//! Solution for the `acyclic-maze` challenge.

#![cfg_attr(target_arch = "riscv32", no_std, no_main)]

extern crate kartoffel;

use kartoffel::*;

#[cfg_attr(target_arch = "riscv32", no_mangle)]
fn main() {
    loop {
        radar_wait();
        radar_scan(3);

        if radar_read(0, -1) == '@' {
            arm_stab();
            break;
        }

        if radar_read(0, -1) == '.' {
            if radar_read(1, -1) == '-' || radar_read(1, -1) == '|' {
                motor_wait();
                motor_step_fw();
            } else {
                motor_wait();
                motor_step_fw();

                motor_wait();
                motor_turn_right();

                motor_wait();
                motor_step_fw();
            }
        } else {
            motor_wait();
            motor_turn_left();
        }
    }
}

//! Solution for the `acyclic-maze` challenge.

#![cfg_attr(target_arch = "riscv32", no_std, no_main)]

extern crate kartoffel;

use kartoffel::*;

#[cfg_attr(target_arch = "riscv32", no_mangle)]
fn main() {
    loop {
        let scan = {
            radar_wait();
            radar_scan_3x3()
        };

        if scan.at(0, -1) == '@' {
            arm_stab();
            break;
        }

        if scan.at(0, -1) == '.' {
            if scan.at(1, -1) == '-' || scan.at(1, -1) == '|' {
                motor_wait();
                motor_step();
            } else {
                motor_wait();
                motor_step();

                motor_wait();
                motor_turn_right();

                motor_wait();
                motor_step();
            }
        } else {
            motor_wait();
            motor_turn_left();
        }
    }
}

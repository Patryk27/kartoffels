#![cfg_attr(target_arch = "riscv64", no_std, no_main)]

extern crate kartoffel;

use kartoffel::*;

#[cfg_attr(target_arch = "riscv64", no_mangle)]
fn main() {
    loop {
        let scan = {
            radar_wait();
            radar_scan_3x3()
        };

        if is_bot(scan[0][1]) {
            arm_stab();

            loop {
                //
            }
        }

        if is_floor(scan[0][1]) {
            if is_wall(scan[0][2]) {
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

fn is_bot(ch: char) -> bool {
    ch == '@'
}

fn is_floor(ch: char) -> bool {
    ch == '.'
}

fn is_wall(ch: char) -> bool {
    ch == '-' || ch == '|'
}

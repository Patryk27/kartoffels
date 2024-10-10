#![cfg_attr(target_arch = "riscv64", no_std, no_main)]

use kartoffel::*;

#[cfg_attr(target_arch = "riscv64", no_mangle)]
fn main() {
    loop {
        radar_wait();

        let scan = radar_scan_3x3();

        if scan[0][1] == '.' {
            motor_wait();
            motor_step();
        } else if scan[1][0] == '.' {
            motor_wait();
            motor_turn_left();
        } else if scan[1][2] == '.' {
            motor_wait();
            motor_turn_right();
        }
    }
}

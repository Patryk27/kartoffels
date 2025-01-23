//! Solution for the `diamond-heist` challenge.

#![cfg_attr(target_arch = "riscv64", no_std, no_main)]

extern crate kartoffel;

use kartoffel::*;

#[cfg_attr(target_arch = "riscv64", no_mangle)]
fn main() {
    serial_write("driving to entrance\n");

    for _ in 0..9 {
        motor_wait();
        motor_step();
    }

    // ---

    serial_write("waiting for guard\n");

    loop {
        let scan = {
            radar_wait();
            radar_scan_5x5()
        };

        if scan.at(0, -2) == '@' {
            break;
        }
    }

    // ---

    serial_write("blending in\n");

    let ticks = timer_ticks();

    while timer_ticks() < ticks + 32000 {
        //
    }

    for _ in 0..2 {
        motor_wait();
        motor_step();
    }

    motor_wait();
    motor_turn_left();

    for _ in 0..3 {
        motor_wait();
        motor_step();
    }

    motor_wait();
    motor_turn_right();

    for _ in 0..3 {
        motor_wait();
        motor_step();
    }

    // ---

    serial_write("picking diamond\n");
    arm_pick();

    // ---

    serial_write("blending in\n");

    motor_wait();
    motor_turn_left();

    for _ in 0..2 {
        motor_wait();
        motor_step();
    }

    loop {
        let scan = {
            radar_wait();
            radar_scan_5x5()
        };

        if scan.at(1, -2) == '@' {
            break;
        }
    }

    motor_wait();
    motor_step();

    motor_wait();
    motor_turn_right();

    for _ in 0..3 {
        motor_wait();
        motor_step();
    }

    loop {
        let scan = {
            radar_wait();
            radar_scan_5x5()
        };

        if scan.at(1, -2) == '@' {
            break;
        }
    }

    // ---

    serial_write("running to the exit\n");

    loop {
        motor_wait();
        motor_step();
    }
}

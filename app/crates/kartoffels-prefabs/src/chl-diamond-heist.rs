//! Solution for the `diamond-heist` challenge.

#![no_std]
#![no_main]

extern crate kartoffel;

use kartoffel::*;

#[no_mangle]
fn main() {
    println!("driving to entrance");

    for _ in 0..9 {
        motor_wait();
        motor_step_fw();
    }

    // ---

    println!("waiting for guard");

    loop {
        radar_wait();
        radar_scan_ex(5, RADAR_SCAN_BOTS);

        if radar_read(0, -2) == '@' {
            break;
        }
    }

    // ---

    println!("blending in");

    let ticks = timer_ticks();

    while timer_ticks() < ticks + 32000 {
        //
    }

    for _ in 0..2 {
        motor_wait();
        motor_step_fw();
    }

    motor_wait();
    motor_turn_left();

    for _ in 0..3 {
        motor_wait();
        motor_step_fw();
    }

    motor_wait();
    motor_turn_right();

    for _ in 0..3 {
        motor_wait();
        motor_step_fw();
    }

    // ---

    println!("picking diamond");
    arm_pick();

    // ---

    println!("blending in");

    motor_wait();
    motor_turn_left();

    for _ in 0..2 {
        motor_wait();
        motor_step_fw();
    }

    loop {
        radar_wait();
        radar_scan_ex(5, RADAR_SCAN_BOTS);

        if radar_read(1, -2) == '@' {
            break;
        }
    }

    motor_wait();
    motor_step_fw();

    motor_wait();
    motor_turn_right();

    for _ in 0..3 {
        motor_wait();
        motor_step_fw();
    }

    loop {
        radar_wait();
        radar_scan_ex(5, RADAR_SCAN_BOTS);

        if radar_read(1, -2) == '@' {
            break;
        }
    }

    // ---

    println!("running to the exit");

    loop {
        motor_wait();
        motor_step_fw();
    }
}

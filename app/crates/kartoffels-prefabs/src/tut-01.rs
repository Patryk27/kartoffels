#![no_std]
#![no_main]

use kartoffel::*;

#[no_mangle]
fn main() {
    loop {
        motor_wait();
        motor_step();

        motor_wait();
        motor_step();

        motor_wait();
        motor_step();

        motor_wait();
        motor_turn_right();
    }
}

#![no_std]
#![no_main]

use kartoffel::*;

#[no_mangle]
fn main() {
    loop {
        motor_step_fw();
    }
}

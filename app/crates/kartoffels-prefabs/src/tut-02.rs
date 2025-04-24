//! Bot used for tutorial's tests.

#![no_std]
#![no_main]

use kartoffel::*;

#[no_mangle]
fn main() {
    loop {
        motor_wait();
        motor_step_fw();
    }
}

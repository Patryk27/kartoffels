//! Dummy bot, does nothing.

#![no_std]
#![no_main]

extern crate kartoffel;

#[no_mangle]
fn main() {
    #[allow(clippy::empty_loop)]
    loop {
        //
    }
}

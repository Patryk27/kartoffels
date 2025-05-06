#![no_std]
#![no_main]

use kartoffel::*;

#[no_mangle]
fn main() {
    println!("Hello, World!");

    loop {
        println!("{}", clock_ticks());
    }
}

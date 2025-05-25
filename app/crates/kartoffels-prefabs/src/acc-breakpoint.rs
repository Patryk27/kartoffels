#![no_std]
#![no_main]

use kartoffel::*;

#[unsafe(no_mangle)]
fn main() {
    print!("one ");
    breakpoint();
    print!("two ");
    breakpoint();
    print!("three ");
}

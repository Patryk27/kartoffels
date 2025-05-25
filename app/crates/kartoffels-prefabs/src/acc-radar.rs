#![no_std]
#![no_main]

use kartoffel::*;

#[unsafe(no_mangle)]
fn main() {
    radar_wait();
    radar_scan(5);

    for y in -2..=2 {
        for x in -2..=2 {
            print!("{}", radar_read(x, y));
        }

        println!();
    }
}

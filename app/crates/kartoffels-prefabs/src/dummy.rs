//! Dummy bot, does nothing.

#![cfg_attr(target_arch = "riscv32", no_std, no_main)]

extern crate kartoffel;

#[cfg_attr(target_arch = "riscv32", no_mangle)]
fn main() {
    #[allow(clippy::empty_loop)]
    loop {
        //
    }
}

//! Dummy bot, does nothing.

#![cfg_attr(target_arch = "riscv64", no_std, no_main)]

extern crate kartoffel;

#[cfg_attr(target_arch = "riscv64", no_mangle)]
fn main() {
    #[allow(clippy::empty_loop)]
    loop {
        //
    }
}

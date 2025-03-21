//! Ianthe - Sender bot in the Tridentarius pair currently used for testing sending / receiving bluetooth messages

#![cfg_attr(target_arch = "riscv32", no_std, no_main)]

extern crate alloc;

use alloc::collections::VecDeque;

use kartoffel::*;

#[cfg_attr(target_arch = "riscv32", no_mangle)]
fn main() {
    let mut display = Display::default();
    let mut counter: u8 = 65;
    loop {
        let message: [u8; 32] = [counter; 32];
        display.log(counter as char);
        bluetooth_wait();
        bluetooth_send_9x9(message);
        counter += 1;
        if counter >= 126 {
            counter = 65
        }
    }
}
#[derive(Default)]
struct Display {
    logs: VecDeque<char>,
}

impl Display {
    fn log(&mut self, log: char) {
        if self.logs.len() >= 3 * 30 {
            self.logs.pop_front();
        }

        self.logs.push_back(log);
        self.send();
    }

    fn send(&self) {
        serial_buffer();

        println!("i'm ianthe \n");

        for log in &self.logs {
            print!("{}", *log);
        }

        serial_flush();
    }
}

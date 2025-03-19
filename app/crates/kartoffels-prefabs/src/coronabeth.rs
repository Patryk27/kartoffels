//! coronabeth - reciever bot in the Tridentarius pair currently used for testing sending / recieving bluetooth messages

#![cfg_attr(target_arch = "riscv32", no_std, no_main)]

extern crate alloc;

use alloc::collections::VecDeque;

use kartoffel::*;

#[cfg_attr(target_arch = "riscv32", no_mangle)]
fn main() {
    let mut display = Display::default();
    loop {
        if let Some(message) = pop_bluetooth_buffer() {
            // check that each value is the same
            match unique(message.message) {
                Some(v) => {
                    display.log(v as char);
                }
                None => {
                    ['E', 'R', 'R', 'O', 'R']
                        .iter()
                        .for_each(|&v| display.log(v));
                }
            }
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

        println!("i'm Coronabeth \n");

        for log in &self.logs {
            print!("{}", *log);
        }

        serial_flush();
    }
}

fn unique<T>(iter: T) -> Option<T::Item>
where
    T: IntoIterator,
    T::Item: Eq,
{
    let mut iter = iter.into_iter();
    let first = iter.next()?;

    iter.all(|item| item == first).then_some(first)
}

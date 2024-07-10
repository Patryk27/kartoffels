#![cfg_attr(target_arch = "riscv64", no_std, no_main)]

extern crate alloc;

use alloc::collections::VecDeque;
use core::ops::Range;
use kartoffel::*;

#[cfg_attr(target_arch = "riscv64", no_mangle)]
fn main() {
    let mut rng = Rng::new();
    let mut display = Display::default();
    let mut sample_dir_at = 0;

    loop {
        display.log('.');

        let scan = {
            radar_wait();
            radar_scan::<5>()
        };

        if scan[1][2] == '@' {
            display.log('!');
            arm_wait();
            arm_stab();
            continue;
        }

        if scan[1][2] == '.' && got_enemy_in(scan, 0..5, 0..2) {
            display.log('^');
            motor_wait();
            motor_step();
            continue;
        }

        if got_enemy_in(scan, 3..5, 0..5) {
            display.log('<');
            motor_wait();
            motor_turn(1);
            continue;
        }

        if got_enemy_in(scan, 0..2, 0..5) {
            display.log('>');
            motor_wait();
            motor_turn(-1);
            continue;
        }

        if got_enemy_in(scan, 0..5, 3..5) {
            display.log('v');
            motor_wait();
            motor_turn(if rng.bool() { -1 } else { 1 });
            continue;
        }

        if timer_ticks() < sample_dir_at && scan[1][2] != '.' {
            sample_dir_at = 0;
        }

        if timer_ticks() < sample_dir_at {
            motor_wait();
            motor_step();
        } else {
            display.log('?');

            let can_step = loop {
                match rng.u32() % 4 {
                    0 if scan[2][1] == '.' => {
                        motor_wait();
                        motor_turn(-1);

                        break true;
                    }

                    1 if scan[1][2] == '.' => {
                        break true;
                    }

                    2 if scan[2][3] == '.' => {
                        motor_wait();
                        motor_turn(1);

                        break true;
                    }

                    3 if scan[3][2] == '.' => {
                        motor_wait();
                        motor_turn(if rng.bool() { -1 } else { 1 });

                        break false;
                    }

                    _ => (),
                }
            };

            if can_step {
                motor_wait();
                motor_step();

                sample_dir_at = timer_ticks() + (rng.u32() % 20) * 8000;
            }
        }
    }
}

fn got_enemy_in<const D: usize>(
    scan: [[char; D]; D],
    xs: Range<usize>,
    ys: Range<usize>,
) -> bool {
    for y in ys {
        for x in xs.clone() {
            if scan[y][x] == '@' {
                return true;
            }
        }
    }

    false
}

#[derive(Clone, Debug)]
struct Rng {
    state: u32,
}

impl Rng {
    fn new() -> Self {
        Self {
            state: timer_seed(),
        }
    }

    fn bool(&mut self) -> bool {
        self.u32() >= u32::MAX / 2
    }

    fn u32(&mut self) -> u32 {
        self.state = 1664525 * self.state + 1013904223;
        self.state
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
        serial_send_ctrl(SerialCtrlChar::StartBuffering);
        serial_send_str("i'm roberto 🔪\n\n");

        for log in &self.logs {
            serial_send(*log);
        }

        serial_send_ctrl(SerialCtrlChar::FlushBuffer);
    }
}

//! Roberto - a moderately challenging bot that likes to stab.

#![cfg_attr(target_arch = "riscv32", no_std, no_main)]

extern crate alloc;

use alloc::collections::VecDeque;
use core::ops::RangeInclusive;
use kartoffel::*;

#[cfg_attr(target_arch = "riscv32", no_mangle)]
fn main() {
    let mut rng = Rng::new();
    let mut display = Display::default();
    let mut sample_dir_at = 0;

    loop {
        display.log('.');

        radar_wait();
        radar_scan(5);

        // If there's an enemy right in front of us, attack, attack, attack!
        if radar_read(0, -1) == '@' {
            display.log('!');
            arm_wait();
            arm_stab();
            continue;
        }

        // If there's an enemy somewhere in front of us, move forward
        if radar_read(0, -1) == '.' && got_enemy_in(-2..=2, -2..=1) {
            display.log('^');
            motor_wait();
            motor_step_fw();
            continue;
        }

        // If there's an enemy somewhere to the left of us, turn left
        if got_enemy_in(-2..=0, -2..=2) {
            display.log('<');
            motor_wait();
            motor_turn_left();
            continue;
        }

        // If there's an enemy somewhere to the right of us, turn right
        if got_enemy_in(0..=2, -2..=2) {
            display.log('>');
            motor_wait();
            motor_turn_right();
            continue;
        }

        // If there's an enemy behind us, turn in random direction (hoping to
        // eventually turn back)
        if got_enemy_in(-2..=2, 1..=2) {
            display.log('v');
            motor_wait();

            if rng.bool() {
                motor_turn_left();
            } else {
                motor_turn_right();
            }

            continue;
        }

        // If the direction we're moving towards will cause us to fall outside
        // the map, change direction
        if timer_ticks() < sample_dir_at && radar_read(0, -1) != '.' {
            sample_dir_at = 0;
        }

        if timer_ticks() < sample_dir_at {
            motor_wait();
            motor_step_fw();
        } else {
            display.log('?');

            let can_step = loop {
                match rng.u32() % 4 {
                    0 if radar_read(-1, 0) == '.' => {
                        motor_wait();
                        motor_turn_left();

                        break true;
                    }

                    1 if radar_read(0, -1) == '.' => {
                        break true;
                    }

                    2 if radar_read(1, 0) == '.' => {
                        motor_wait();
                        motor_turn_right();

                        break true;
                    }

                    3 if radar_read(0, 1) == '.' => {
                        motor_wait();

                        if rng.bool() {
                            motor_turn_left();
                        } else {
                            motor_turn_right();
                        }

                        break false;
                    }

                    _ => (),
                }
            };

            if can_step {
                motor_wait();
                motor_step_fw();

                sample_dir_at = timer_ticks() + (rng.u32() % 20) * 8000;
            }
        }
    }
}

fn got_enemy_in(xs: RangeInclusive<i32>, ys: RangeInclusive<i32>) -> bool {
    for x in xs {
        for y in ys.clone() {
            if radar_read(x, y) == '@' {
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
        serial_buffer();

        println!("i'm roberto 🔪\n");

        for log in &self.logs {
            print!("{}", *log);
        }

        serial_flush();
    }
}

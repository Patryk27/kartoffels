//! Roberto - a moderately challenging bot that likes to stab.

#![cfg_attr(target_arch = "riscv64", no_std, no_main)]

extern crate alloc;

use alloc::collections::VecDeque;
use core::ops::RangeInclusive;
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
            radar_scan_5x5()
        };

        // If there's an enemy right in front of us, attack, attack, attack!
        if scan.at(0, -1) == '@' {
            display.log('!');
            arm_wait();
            arm_stab();
            continue;
        }

        // If there's an enemy somewhere in front of us, move forward
        if scan.at(0, -1) == '.' && got_enemy_in(&scan, -2..=2, -2..=1) {
            display.log('^');
            motor_wait();
            motor_step();
            continue;
        }

        // If there's an enemy somewhere to the left of us, turn left
        if got_enemy_in(&scan, -2..=0, -2..=2) {
            display.log('<');
            motor_wait();
            motor_turn_left();
            continue;
        }

        // If there's an enemy somewhere to the right of us, turn right
        if got_enemy_in(&scan, 0..=2, -2..=2) {
            display.log('>');
            motor_wait();
            motor_turn_right();
            continue;
        }

        // If there's an enemy behind us, turn in random direction (hoping to
        // eventually turn back)
        if got_enemy_in(&scan, -2..=2, 1..=2) {
            display.log('v');
            motor_wait();
            motor_turn(if rng.bool() { -1 } else { 1 });
            continue;
        }

        // If the direction we're moving towards will cause us to fall outside
        // the map, change direction
        if timer_ticks() < sample_dir_at && scan.at(0, -1) != '.' {
            sample_dir_at = 0;
        }

        if timer_ticks() < sample_dir_at {
            motor_wait();
            motor_step();
        } else {
            display.log('?');

            let can_step = loop {
                match rng.u32() % 4 {
                    0 if scan.at(-1, 0) == '.' => {
                        motor_wait();
                        motor_turn_left();

                        break true;
                    }

                    1 if scan.at(0, -1) == '.' => {
                        break true;
                    }

                    2 if scan.at(1, 0) == '.' => {
                        motor_wait();
                        motor_turn_right();

                        break true;
                    }

                    3 if scan.at(0, 1) == '.' => {
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
    scan: &RadarScan<D>,
    xs: RangeInclusive<i8>,
    ys: RangeInclusive<i8>,
) -> bool {
    for x in xs {
        for y in ys.clone() {
            if scan.at(x, y) == '@' {
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
        serial_write(SerialControlCode::StartBuffering);
        serial_write("i'm roberto 🔪\n\n");

        for log in &self.logs {
            serial_write(*log);
        }

        serial_write(SerialControlCode::FlushBuffer);
    }
}

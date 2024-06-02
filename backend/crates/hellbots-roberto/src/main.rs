#![no_std]
#![cfg_attr(target_arch = "riscv64", no_main)]

use core::ops::Range;
use hellbots_pac::*;

init!(+panic);

#[cfg_attr(target_arch = "riscv64", no_mangle)]
fn main() {
    let mut rng = Rng::new();
    let mut sample_dir_at = 0;

    loop {
        uart_send('.');

        let radar = {
            radar_wait();
            radar_scan(5);
            radar_wait();
            radar_read::<5>()
        };

        if radar[1][2] == '@' {
            uart_send('!');
            arm_stab();
            continue;
        }

        if radar[1][2] == '.' && got_enemy_in(radar, 0..5, 0..2) {
            uart_send('^');
            motor_wait();
            motor_step();
            continue;
        }

        if got_enemy_in(radar, 3..5, 0..5) {
            uart_send('<');
            motor_wait();
            motor_turn(1);
            continue;
        }

        if got_enemy_in(radar, 0..2, 0..5) {
            uart_send('>');
            motor_wait();
            motor_turn(-1);
            continue;
        }

        if got_enemy_in(radar, 0..5, 3..5) {
            uart_send('v');
            motor_wait();
            motor_turn(if rng.bool() { -1 } else { 1 });
            continue;
        }

        if timer_ticks() < sample_dir_at && radar[1][2] != '.' {
            uart_send('?');
            sample_dir_at = 0;
        }

        if timer_ticks() < sample_dir_at {
            motor_wait();
            motor_step();
        } else {
            let can_step;

            loop {
                match rng.u32() % 4 {
                    0 if radar[2][1] == '.' => {
                        motor_wait();
                        motor_turn(-1);
                        can_step = true;
                        break;
                    }

                    1 if radar[1][2] == '.' => {
                        can_step = true;
                        break;
                    }

                    2 if radar[2][3] == '.' => {
                        motor_wait();
                        motor_turn(1);
                        can_step = true;
                        break;
                    }

                    3 if radar[3][2] == '.' => {
                        motor_wait();
                        motor_turn(if rng.bool() { -1 } else { 1 });
                        can_step = false;
                        break;
                    }

                    _ => (),
                }
            }

            if can_step {
                motor_wait();
                motor_step();

                sample_dir_at = timer_ticks() + (rng.u32() % 20) * 5000;
            }
        }
    }
}

fn motor_wait() {
    while !is_motor_ready() {
        //
    }
}

fn radar_wait() {
    while !is_radar_ready() {
        //
    }
}

fn got_enemy_in<const R: usize>(
    radar: [[char; R]; R],
    xs: Range<usize>,
    ys: Range<usize>,
) -> bool {
    for y in ys {
        for x in xs.clone() {
            if radar[y][x] == '@' {
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

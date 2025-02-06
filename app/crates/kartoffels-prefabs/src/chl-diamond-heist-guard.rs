//! Guard used for the `diamond-heist` challenge.

#![cfg_attr(target_arch = "riscv32", no_std, no_main)]

extern crate alloc;
extern crate kartoffel;

use alloc::vec::Vec;
use core::num::NonZeroU64;
use kartoffel::*;

#[cfg_attr(target_arch = "riscv32", no_mangle)]
fn main() {
    while timer_ticks() <= 16000 {
        //
    }

    let guards = find_guards();

    for _ in 0..2 {
        motor_wait();
        motor_step_fw();
    }

    loop {
        let scan = {
            radar_wait();
            radar_scan_3x3()
        };

        if let Some((dx, dy)) = find_enemy(&guards, &scan) {
            attack_enemy(dx, dy);
            continue;
        }

        match scan.at(0, -1) {
            '.' => {
                motor_wait();
                motor_step_fw();
            }

            '|' | '-' => {
                motor_wait();
                motor_turn_right();
            }

            _ => (),
        }
    }
}

fn find_guards() -> Vec<NonZeroU64> {
    let scan = radar_scan_5x5();
    let mut guards = Vec::new();

    for dx in -2..=2 {
        for dy in -2..=2 {
            if dx == 1 && dy == 0 {
                continue;
            }

            if let Some(id) = scan.bot_at(dx, dy) {
                guards.push(id);
            }
        }
    }

    guards
}

fn find_enemy(guards: &[NonZeroU64], scan: &RadarScan<3>) -> Option<(i8, i8)> {
    for dx in -1..=1 {
        for dy in -1..=1 {
            if let Some(id) = scan.bot_at(dx, dy) {
                if !guards.contains(&id) {
                    return Some((dx, dy));
                }
            }
        }
    }

    None
}

#[allow(clippy::collapsible_else_if)]
#[allow(clippy::comparison_chain)]
fn attack_enemy(dx: i8, dy: i8) {
    if dy == -1 {
        if dx == 0 {
            if is_arm_ready() {
                arm_stab();
            }
        } else {
            motor_wait();
            motor_step_fw();
        }
    } else {
        if dx < 0 {
            motor_wait();
            motor_turn_left();
        } else if dx > 0 {
            motor_wait();
            motor_turn_right();
        }
    }
}

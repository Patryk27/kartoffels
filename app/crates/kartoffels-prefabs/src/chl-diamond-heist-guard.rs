#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec::Vec;
use kartoffel::*;

#[unsafe(no_mangle)]
fn main() {
    while clock_ticks() <= 16000 {
        //
    }

    let guards = find_guards();

    if guards.len() != 4 {
        panic!("where my friends at?");
    }

    for _ in 0..2 {
        motor_wait();
        motor_step();
    }

    loop {
        radar_wait();
        radar_scan_ex(3, RADAR_SCAN_BOTS | RADAR_SCAN_TILES | RADAR_SCAN_IDS);

        if let Some((dx, dy)) = find_enemy(&guards) {
            attack_enemy(dx, dy);
            continue;
        }

        match radar_read(0, -1) {
            '.' => {
                motor_wait();
                motor_step();
            }

            '|' | '-' => {
                motor_wait();
                motor_turn_right();
            }

            _ => (),
        }
    }
}

fn find_guards() -> Vec<u64> {
    radar_scan_ex(5, RADAR_SCAN_BOTS | RADAR_SCAN_IDS);

    let mut guards = Vec::new();

    for dx in -2..=2 {
        for dy in -2..=2 {
            if radar_read(dx, dy) == '@' {
                guards.push(radar_read_id(dx, dy));
            }
        }
    }

    guards
}

fn find_enemy(guards: &[u64]) -> Option<(i32, i32)> {
    for dx in -1..=1 {
        for dy in -1..=1 {
            if radar_read(dx, dy) == '@'
                && !guards.contains(&radar_read_id(dx, dy))
            {
                return Some((dx, dy));
            }
        }
    }

    None
}

#[allow(clippy::collapsible_else_if)]
#[allow(clippy::comparison_chain)]
fn attack_enemy(dx: i32, dy: i32) {
    if dy == -1 {
        if dx == 0 {
            if arm_ready() {
                arm_stab();
            }
        } else {
            motor_wait();
            motor_step();
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

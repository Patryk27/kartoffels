#![cfg_attr(target_arch = "riscv64", no_std, no_main)]

extern crate alloc;
extern crate kartoffel;

use alloc::vec::Vec;
use core::num::NonZeroU64;
use kartoffel::*;

#[cfg_attr(target_arch = "riscv64", no_mangle)]
fn main() {
    while timer_ticks() <= 64000 {
        //
    }

    let friends = find_friends();

    loop {
        if let Some((ex, ey)) = find_enemy(&friends) {
            if ey == -1 && ex == 0 {
                if is_arm_ready() {
                    arm_stab();
                }

                continue;
            }

            if ey < -1 {
                motor_step();
                continue;
            }

            if ex < 0 {
                motor_turn_left();
            } else if ex > 0 {
                motor_turn_right();
            }
        }
    }
}

fn find_friends() -> Vec<NonZeroU64> {
    let scan = radar_scan_9x9();
    let mut friends = Vec::new();

    for dx in -4..=4 {
        for dy in -4..=4 {
            if let Some(id) = scan.bot_at(dx, dy) {
                friends.push(id);
            }
        }
    }

    friends
}

fn find_enemy(friends: &[NonZeroU64]) -> Option<(i8, i8)> {
    let scan = {
        radar_wait();
        radar_scan_9x9()
    };

    for dx in -4..=4 {
        for dy in -4..=4 {
            if let Some(id) = scan.bot_at(dx, dy) {
                if !friends.contains(&id) {
                    return Some((dx, dy));
                }
            }
        }
    }

    None
}

#![cfg_attr(target_arch = "riscv64", no_std, no_main)]

extern crate kartoffel;

use kartoffel::*;

#[cfg_attr(target_arch = "riscv64", no_mangle)]
fn main() {
    let friends = find_friends();

    loop {
        if let Some((ex, _ey)) = find_enemy(&friends) {
            if ex < 0 {
                motor_turn_left();
            } else if ex > 0 {
                motor_turn_right();
            }
        }
    }
}

fn find_friends() -> Vec<u64> {
    let scan = radar_scan_9x9();
    let mut friends = Vec::new();

    for dx in -4..=4 {
        for dy in -4..=4 {
            let id = scan.bot_at(dx, dy);

            if id > 0 {
                friends.push(id);
            }
        }
    }

    friends
}

fn find_enemy(friends: &[u64]) -> Option<(i8, i8)> {
    let scan = {
        radar_wait();
        radar_scan_3x3()
    };

    for dx in -1..=1 {
        for dy in -1..=1 {
            let id = scan.bot_at(dx, dy);

            if !friends.contains(&id) {
                return Some((dx, dy));
            }
        }
    }

    None
}

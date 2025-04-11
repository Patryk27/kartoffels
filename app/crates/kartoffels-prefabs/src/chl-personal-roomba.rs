//! Solution for the `personal-roomba` challenge.
//!
//! We use an unbiased searching algorithm (i.e. we don't assume that flags are
//! located in the corners) that boils down to:
//!
//! ```text
//! loop:
//!     scan_terrain()
//!     path = find_closest_unknown_tile_or_flag()
//!     go_to(path[0])
//! ```
//!
//! ... where `find_closest_unknown_tile_or_flag()` is realized through the
//! Dijkstra algorithm.
//!
//! Overall, this algorithm causes the bot to search the entire maze, taking
//! into account loops and whatnot.
//!
//! The actual implementation is a bit convoluted, since we implement a couple
//! of performance tricks:
//!
//! - we don't scan the terrain if we already know all the surrounding area,
//!
//! - we recalculate the path only if our knowledge of the map changed (or the
//!   previous path's goal has been reached).
//!
//! We could do even more fancy tricks - e.g. currently the path-finder always
//! assumes that moving has a cost=1, which is not really true (e.g. reversing
//! the bot has a higher cost than moving forward); but it's good enough.

#![cfg_attr(target_arch = "riscv32", no_std, no_main)]

extern crate alloc;

use alloc::collections::binary_heap::BinaryHeap;
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::iter;
use core::ops::Add;
use glam::{i8vec2, I8Vec2};
use kartoffel::*;

#[cfg_attr(target_arch = "riscv32", no_mangle)]
fn main() {
    // The World as we know it.
    //
    // We start with all tiles set to `Tile::Unknown` and gradually learn the
    // map as we march through it.
    let mut map = Map::new();

    // Our position, relative to the spawnpoint (hence it can be negative)
    let mut pos = I8Vec2::default();

    // Our direction, absolute
    let mut dir = match compass_dir() {
        1 => Dir::N,
        2 => Dir::E,
        3 => Dir::S,
        4 => Dir::W,
        _ => unreachable!(),
    };

    // Path towards the closest `Tile::Unknown` or `Tile::Find` as found by the
    // pathfinding algorithm.
    //
    // It's a list of points where the last point (i.e. `nav_path.pop()`) gives
    // us the closest step we should march towards.
    let mut nav_path = Vec::with_capacity(32);

    // Queue of nodes left to process, used by the pathfinding algorithm.
    //
    // It's a temporary structure used by `navigate()` - we keep it alive inside
    // `main()` to reduce pathfinding's memory pressure.
    let mut nav_queue = BinaryHeap::with_capacity(256);

    // Cost of moving between nodes, used by the pathfinding algorithm.
    //
    // Same case as `nav_queue`, we keep it here for performance reasons.
    let mut nav_costs: Vec<_> = iter::repeat(0).take(map.tiles.len()).collect();

    // Parent nodes, used by the pathfinding algorithm to reconstruct the path
    // after we find a `Tile::Unknown` or `Tile::Flag`.
    //
    // Same case as `nav_queue`, we keep it here for performance reasons.
    let mut nav_prevs: Vec<_> = iter::repeat(I8Vec2::default())
        .take(map.tiles.len())
        .collect();

    loop {
        // 1. Scan the environment - if something has changed, reset our cached
        //    path.
        if scan(&mut map, pos, dir) {
            nav_path.clear();
        }

        // 2. If there's no path, call the pathfinder to find us the closest
        //    unknown tile or flag.
        if nav_path.is_empty() {
            navigate(
                &mut nav_path,
                &mut nav_queue,
                &mut nav_costs,
                &mut nav_prevs,
                &map,
                pos,
            );
        }

        // 2a. If there's nothing left to find, call it a day.
        let Some(next) = nav_path.pop() else {
            #[allow(clippy::empty_loop)]
            loop {}
        };

        // 3. Drive towards the next point on our path; if there's a flag there,
        //    pick it.
        drive(&mut pos, &mut dir, next);
        pick(&mut map, pos);

        println!();
    }
}

fn scan(map: &mut Map, pos: I8Vec2, dir: Dir) -> bool {
    // Check whether it makes sense to run the radar.
    //
    // If we already know all of the surrounding area, don't bother rescanning
    // it - we know it couldn't have changed since the map is constant here.
    let mut is_scan_required = false;

    for x in -4..=4 {
        for y in -4..=4 {
            let off = I8Vec2 { x, y };

            if map.get(pos + off) == Tile::Unknown {
                is_scan_required = true;
                break;
            }
        }
    }

    if !is_scan_required {
        return false;
    }

    // ---

    print!("scn:");

    radar_wait();
    radar_scan_ex(9, RADAR_SCAN_TILES | RADAR_SCAN_OBJS);

    // Number of discovered tiles
    let mut discovered = 0;

    for x in -4..=4 {
        for y in -4..=4 {
            // In order to store tiles into `map`, we have to to convert the
            // position from our bot-local coordinate system into world-absolute
            // coordinate system.
            //
            // That is, we know that `.scan_at(0, -1)` points at the tile in
            // front of us - but what's the absolute position of that tile?
            //
            // Intuitively if we're at `(2, 5)` and:
            //
            // - if we're looking at `Dir::N`, then `.scan_at(0, -1)`
            //   corresponds to tile `(2, 4)`,
            //
            // - if we're looking at `Dir::S`, then `.scan_at(0, -1)`
            //   corresponds to tile `(2, 6)`,
            //
            // - if we're looking at `Dir::E`, then `.scan_at(0, -1)`
            //   corresponds to tile `(3, 5)`.
            //
            // - etc. etc.
            //
            // Note that we don't actually know "absolute-absolute" coordinates,
            // we can only speak in coordinates relative to the spawnpoint -
            // that's what I refer to here.
            let pos = {
                let off = I8Vec2 { x, y };
                let off = dir.as_vec().rotate(off.perp());

                I8Vec2 {
                    x: pos.x + off.x,
                    y: pos.y + off.y,
                }
            };

            let tile = match radar_read(x as i32, y as i32) {
                '.' | '@' => Tile::Floor,
                '=' => Tile::Flag,
                _ => Tile::WallOrVoid,
            };

            if map.set(pos, tile) {
                discovered += 1;
            }
        }
    }

    print!("{discovered} ");

    discovered > 0
}

fn navigate(
    path: &mut Vec<I8Vec2>,
    queue: &mut BinaryHeap<NavEntry>,
    costs: &mut [u8],
    prevs: &mut [I8Vec2],
    map: &Map,
    head: I8Vec2,
) -> bool {
    print!("nav:");

    queue.clear();
    queue.push(NavEntry { cost: 0, pos: head });

    costs.fill(u8::MAX);
    costs[map.idx(head)] = 0;

    while let Some(NavEntry { cost, mut pos }) = queue.pop() {
        if cost > costs[map.idx(pos)] {
            continue;
        }

        match map.get(pos) {
            Tile::Flag | Tile::Unknown => {
                print!("{cost} ");

                while pos != head {
                    path.push(pos);
                    pos = prevs[map.idx(pos)];
                }

                return true;
            }

            _ => {
                let prev = pos;
                let cost = cost + 1;

                for dir in Dir::all() {
                    let pos = pos + dir;

                    if map.get(pos) != Tile::WallOrVoid
                        && cost < costs[map.idx(pos)]
                    {
                        queue.push(NavEntry { cost, pos });

                        costs[map.idx(pos)] = cost;
                        prevs[map.idx(pos)] = prev;
                    }
                }
            }
        }
    }

    print!("-");

    false
}

fn drive(pos: &mut I8Vec2, dir: &mut Dir, target: I8Vec2) {
    print!("drv:");

    // Convert `target` from position (as generated by the pathfinder) into a
    // direction (as required by us to actually move the bot).
    //
    // Assuming that the pathfinder works correctly, `target` should be at most
    // one tile away from us, and it shouldn't be on the diagonal, so we have to
    // solve this:
    //
    //     pos + dir == target
    //
    // ... for `dir` -- which is pretty simple, considering that `dir` has only
    //     four possible values (brute-force, brute-force, brute-force!)
    let target = Dir::all().find(|dir| *pos + *dir == target).unwrap();

    match (*dir, target) {
        (Dir::N, Dir::N)
        | (Dir::E, Dir::E)
        | (Dir::S, Dir::S)
        | (Dir::W, Dir::W) => {
            print!("^ ");

            motor_wait();
            motor_step_fw();
        }

        (Dir::N, Dir::W)
        | (Dir::W, Dir::S)
        | (Dir::S, Dir::E)
        | (Dir::E, Dir::N) => {
            print!("< ");

            motor_wait();
            motor_turn_left();
            motor_wait();
            motor_step_fw();
        }

        (Dir::N, Dir::E)
        | (Dir::E, Dir::S)
        | (Dir::S, Dir::W)
        | (Dir::W, Dir::N) => {
            print!("> ");

            motor_wait();
            motor_turn_right();
            motor_wait();
            motor_step_fw();
        }

        (Dir::N, Dir::S)
        | (Dir::E, Dir::W)
        | (Dir::S, Dir::N)
        | (Dir::W, Dir::E) => {
            print!("v ");

            motor_wait();
            motor_turn_left();
            motor_wait();
            motor_turn_left();
            motor_wait();
            motor_step_fw();
        }
    }

    *pos = *pos + target;
    *dir = target;
}

fn pick(map: &mut Map, pos: I8Vec2) -> bool {
    if map.get(pos) == Tile::Flag {
        print!("flg!");

        arm_wait();
        arm_pick();

        // Pathfinder treats the flag as a passable tile and expects us to
        // actually walk there, so let's do that.
        //
        // Alternatively we could change the logic inside `drive()` so that we
        // don't do `*pos = *pos + target;` if `target` points at a flag, but
        // it's easier to actually just walk there.
        motor_wait();
        motor_step_fw();

        map.set(pos, Tile::Floor);

        true
    } else {
        false
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Dir {
    N,
    E,
    S,
    W,
}

impl Dir {
    fn all() -> impl Iterator<Item = Self> {
        [Dir::N, Dir::E, Dir::W, Dir::S].into_iter()
    }

    fn as_vec(&self) -> I8Vec2 {
        match self {
            Dir::N => i8vec2(0, -1),
            Dir::E => i8vec2(1, 0),
            Dir::S => i8vec2(0, 1),
            Dir::W => i8vec2(-1, 0),
        }
    }
}

impl Add<Dir> for I8Vec2 {
    type Output = I8Vec2;

    fn add(mut self, rhs: Dir) -> Self::Output {
        let rhs = rhs.as_vec();

        self.x += rhs.x;
        self.y += rhs.y;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Tile {
    Unknown,
    Flag,
    Floor,
    WallOrVoid,
}

#[derive(Default)]
struct Map {
    tiles: Vec<Tile>,
}

impl Map {
    const MIN: I8Vec2 = i8vec2(-50, -50);
    const MAX: I8Vec2 = i8vec2(50, 50);

    fn new() -> Self {
        let tiles = (Self::MAX - Self::MIN).as_uvec2();

        let tiles = iter::repeat(Tile::Unknown)
            .take((tiles.x * tiles.y) as usize)
            .collect();

        Self { tiles }
    }

    fn set(&mut self, pos: I8Vec2, tile: Tile) -> bool {
        let idx = self.idx(pos);
        let curr_tile = &mut self.tiles[idx];

        if tile == *curr_tile {
            false
        } else {
            *curr_tile = tile;
            true
        }
    }

    fn get(&self, pos: I8Vec2) -> Tile {
        self.tiles[self.idx(pos)]
    }

    fn idx(&self, pos: I8Vec2) -> usize {
        let pos = pos - Self::MIN;
        let size = Self::MAX - Self::MIN;

        if pos.x >= 0 && pos.y >= 0 && pos.x < size.x && pos.y < size.y {
            (pos.y as usize) * (size.x as usize) + (pos.x as usize)
        } else {
            panic!();
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct NavEntry {
    cost: u8,
    pos: I8Vec2,
}

impl Ord for NavEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .cost
            .cmp(&self.cost)
            .then_with(|| self.pos.x.cmp(&other.pos.x))
            .then_with(|| self.pos.y.cmp(&other.pos.y))
    }
}

impl PartialOrd for NavEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

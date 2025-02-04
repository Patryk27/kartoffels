use crate::MapBuilder;
use anyhow::Result;
use glam::{ivec2, u8vec2, U8Vec2, UVec2};
use itertools::Itertools;
use rand::{Rng, RngCore};

pub async fn wfc(
    rng: &mut impl RngCore,
    map: &mut MapBuilder,
    size: UVec2,
) -> Result<()> {
    let rules = dungeon();

    let mut tiles = vec![
        PendingTile {
            rules: ((1 << (rules.len() + 1)) - 1) as u128,
        };
        (size.x * size.y) as usize
    ];

    map.init(size);
    map.update(|map| map.fill(0));

    loop {
        let tile = tiles
            .iter()
            .enumerate()
            .filter_map(|(idx, tile)| {
                if tile.rules > 0 {
                    Some((idx, tile.rules))
                } else {
                    None
                }
            })
            .sorted_by_key(|(_, rules)| rules.count_ones())
            .next();

        let Some((tile, _)) = tile else {
            break;
        };

        let pos = {
            let x = (tile as u32) % size.x;
            let y = (tile as u32) / size.x;

            u8vec2(x as u8, y as u8)
        };

        loop {
            if tiles[tile].rules == 0 {
                todo!("backtracking required");
            }

            let rule = loop {
                let rule = rng.gen_range(0..128);

                if tiles[tile].rules & (1 << rule) > 0 {
                    break rule;
                }
            };

            if rules[rule].matches(&map, pos) {
                tiles[tile].rules = 0;
                map.set(pos.as_ivec2(), rules[rule].0[4]).await;
                break;
            } else {
                tiles[tile].rules &= !(1 << rule);
            }
        }

        for dy in -1..=1 {
            for dx in -1..=1 {
                let pos = pos.as_ivec2() + ivec2(dx, dy);

                if (dx == 0 && dy == 0) || !map.contains(pos) {
                    continue;
                }

                let idx =
                    (pos.y as usize) * (size.x as usize) + (pos.x as usize);

                let tile = &mut tiles[idx];

                for rule in 0..128 {
                    if tile.rules & (1 << rule) > 0 {
                        if !rules[rule].matches(&map, pos.as_u8vec2()) {
                            tile.rules &= !(1 << rule);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Rule([u8; 9]);

impl Rule {
    fn matches(&self, map: &MapBuilder, pos: U8Vec2) -> bool {
        let mut idx = 0;

        for dy in -1..=1 {
            for dx in -1..=1 {
                let pos = pos.as_ivec2() + ivec2(dx, dy);

                if (dx == 0 && dy == 0) || !map.contains(pos) {
                    idx += 1;
                    continue;
                }

                let act = map.get(pos).kind;
                let exp = self.0[idx];

                if act != 0 && act != exp {
                    return false;
                }

                idx += 1;
            }
        }

        true
    }
}

#[derive(Clone, Copy, Debug)]
struct PendingTile {
    rules: u128,
}

fn dungeon() -> Vec<Rule> {
    let mut rules = Vec::new();

    rules.push(Rule([b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ']));
    rules.push(Rule([b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b'.']));
    rules.push(Rule([b' ', b' ', b' ', b' ', b' ', b' ', b' ', b'.', b'.']));
    rules.push(Rule([b' ', b' ', b' ', b' ', b' ', b' ', b'.', b' ', b' ']));
    rules.push(Rule([b' ', b' ', b' ', b' ', b' ', b' ', b'.', b'.', b' ']));
    rules.push(Rule([b' ', b' ', b' ', b' ', b' ', b' ', b'.', b'.', b'.']));
    rules.push(Rule([b' ', b' ', b' ', b' ', b' ', b'.', b' ', b' ', b'.']));
    rules.push(Rule([b' ', b' ', b' ', b' ', b' ', b'.', b'.', b'.', b'.']));
    rules.push(Rule([b' ', b' ', b' ', b' ', b'.', b'.', b' ', b'.', b'.']));
    rules.push(Rule([b' ', b' ', b' ', b' ', b'.', b'.', b'.', b'.', b'.']));
    rules.push(Rule([b' ', b' ', b' ', b'.', b' ', b' ', b'.', b' ', b' ']));
    rules.push(Rule([b' ', b' ', b' ', b'.', b' ', b' ', b'.', b'.', b'.']));
    rules.push(Rule([b' ', b' ', b' ', b'.', b'.', b' ', b'.', b'.', b' ']));
    rules.push(Rule([b' ', b' ', b' ', b'.', b'.', b' ', b'.', b'.', b'.']));
    rules.push(Rule([b' ', b' ', b' ', b'.', b'.', b'.', b' ', b' ', b' ']));
    rules.push(Rule([b' ', b' ', b' ', b'.', b'.', b'.', b' ', b' ', b'.']));
    rules.push(Rule([b' ', b' ', b' ', b'.', b'.', b'.', b' ', b'.', b'.']));
    rules.push(Rule([b' ', b' ', b' ', b'.', b'.', b'.', b'.', b' ', b' ']));
    rules.push(Rule([b' ', b' ', b' ', b'.', b'.', b'.', b'.', b'.', b' ']));
    rules.push(Rule([b' ', b' ', b' ', b'.', b'.', b'.', b'.', b'.', b'.']));
    rules.push(Rule([b' ', b' ', b'.', b' ', b' ', b' ', b' ', b' ', b' ']));
    rules.push(Rule([b' ', b' ', b'.', b' ', b' ', b'.', b' ', b' ', b' ']));
    rules.push(Rule([b' ', b' ', b'.', b' ', b' ', b'.', b' ', b' ', b'.']));
    rules.push(Rule([b' ', b' ', b'.', b' ', b' ', b'.', b' ', b'.', b'.']));
    rules.push(Rule([b' ', b' ', b'.', b' ', b' ', b'.', b'.', b' ', b'.']));
    rules.push(Rule([b' ', b' ', b'.', b' ', b' ', b'.', b'.', b'.', b'.']));
    rules.push(Rule([b' ', b' ', b'.', b' ', b'.', b'.', b' ', b'.', b'.']));
    rules.push(Rule([b' ', b' ', b'.', b' ', b'.', b'.', b'.', b'.', b'.']));
    rules.push(Rule([b' ', b' ', b'.', b'.', b' ', b'.', b'.', b' ', b'.']));
    rules.push(Rule([b' ', b' ', b'.', b'.', b'.', b'.', b' ', b' ', b' ']));
    rules.push(Rule([b' ', b' ', b'.', b'.', b'.', b'.', b' ', b' ', b'.']));
    rules.push(Rule([b' ', b' ', b'.', b'.', b'.', b'.', b' ', b'.', b'.']));
    rules.push(Rule([b' ', b' ', b'.', b'.', b'.', b'.', b'.', b'.', b'.']));
    rules.push(Rule([b' ', b'.', b' ', b' ', b'.', b' ', b' ', b'.', b' ']));
    rules.push(Rule([b' ', b'.', b' ', b' ', b'.', b' ', b' ', b'.', b'.']));
    rules.push(Rule([b' ', b'.', b' ', b' ', b'.', b' ', b'.', b'.', b'.']));
    rules.push(Rule([b' ', b'.', b' ', b' ', b'.', b'.', b' ', b'.', b'.']));
    rules.push(Rule([b' ', b'.', b' ', b' ', b'.', b'.', b'.', b'.', b'.']));
    rules.push(Rule([b' ', b'.', b' ', b'.', b'.', b'.', b' ', b'.', b' ']));
    rules.push(Rule([b' ', b'.', b' ', b'.', b'.', b'.', b'.', b'.', b'.']));
    rules.push(Rule([b' ', b'.', b'.', b' ', b' ', b' ', b' ', b' ', b' ']));
    rules.push(Rule([b' ', b'.', b'.', b' ', b' ', b'.', b' ', b' ', b'.']));
    rules.push(Rule([b' ', b'.', b'.', b' ', b'.', b' ', b' ', b'.', b' ']));
    rules.push(Rule([b' ', b'.', b'.', b' ', b'.', b'.', b' ', b' ', b' ']));
    rules.push(Rule([b' ', b'.', b'.', b' ', b'.', b'.', b' ', b' ', b'.']));
    rules.push(Rule([b' ', b'.', b'.', b' ', b'.', b'.', b' ', b'.', b' ']));
    rules.push(Rule([b' ', b'.', b'.', b' ', b'.', b'.', b' ', b'.', b'.']));
    rules.push(Rule([b' ', b'.', b'.', b' ', b'.', b'.', b'.', b'.', b'.']));
    rules.push(Rule([b' ', b'.', b'.', b'.', b'.', b'.', b' ', b' ', b' ']));
    rules.push(Rule([b' ', b'.', b'.', b'.', b'.', b'.', b' ', b'.', b'.']));
    rules.push(Rule([b' ', b'.', b'.', b'.', b'.', b'.', b'.', b'.', b'.']));
    rules.push(Rule([b'.', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ']));
    rules.push(Rule([b'.', b' ', b' ', b'.', b' ', b' ', b' ', b' ', b' ']));
    rules.push(Rule([b'.', b' ', b' ', b'.', b' ', b' ', b'.', b' ', b' ']));
    rules.push(Rule([b'.', b' ', b' ', b'.', b' ', b' ', b'.', b' ', b'.']));
    rules.push(Rule([b'.', b' ', b' ', b'.', b' ', b' ', b'.', b'.', b' ']));
    rules.push(Rule([b'.', b' ', b' ', b'.', b' ', b' ', b'.', b'.', b'.']));
    rules.push(Rule([b'.', b' ', b' ', b'.', b' ', b'.', b'.', b' ', b'.']));
    rules.push(Rule([b'.', b' ', b' ', b'.', b'.', b' ', b'.', b'.', b' ']));
    rules.push(Rule([b'.', b' ', b' ', b'.', b'.', b' ', b'.', b'.', b'.']));
    rules.push(Rule([b'.', b' ', b' ', b'.', b'.', b'.', b' ', b' ', b' ']));
    rules.push(Rule([b'.', b' ', b' ', b'.', b'.', b'.', b'.', b' ', b' ']));
    rules.push(Rule([b'.', b' ', b' ', b'.', b'.', b'.', b'.', b'.', b'.']));
    rules.push(Rule([b'.', b' ', b'.', b' ', b' ', b'.', b' ', b' ', b'.']));
    rules.push(Rule([b'.', b' ', b'.', b'.', b' ', b' ', b'.', b' ', b' ']));
    rules.push(Rule([b'.', b' ', b'.', b'.', b' ', b'.', b' ', b' ', b'.']));
    rules.push(Rule([b'.', b' ', b'.', b'.', b' ', b'.', b'.', b' ', b' ']));
    rules.push(Rule([b'.', b' ', b'.', b'.', b' ', b'.', b'.', b' ', b'.']));
    rules.push(Rule([b'.', b' ', b'.', b'.', b' ', b'.', b'.', b'.', b'.']));
    rules.push(Rule([b'.', b' ', b'.', b'.', b'.', b'.', b'.', b'.', b'.']));
    rules.push(Rule([b'.', b'.', b' ', b' ', b' ', b' ', b' ', b' ', b' ']));
    rules.push(Rule([b'.', b'.', b' ', b'.', b' ', b' ', b'.', b' ', b' ']));
    rules.push(Rule([b'.', b'.', b' ', b'.', b'.', b' ', b' ', b' ', b' ']));
    rules.push(Rule([b'.', b'.', b' ', b'.', b'.', b' ', b'.', b' ', b' ']));
    rules.push(Rule([b'.', b'.', b' ', b'.', b'.', b' ', b'.', b'.', b' ']));
    rules.push(Rule([b'.', b'.', b' ', b'.', b'.', b' ', b'.', b'.', b'.']));
    rules.push(Rule([b'.', b'.', b' ', b'.', b'.', b'.', b' ', b' ', b' ']));
    rules.push(Rule([b'.', b'.', b' ', b'.', b'.', b'.', b'.', b'.', b' ']));
    rules.push(Rule([b'.', b'.', b' ', b'.', b'.', b'.', b'.', b'.', b'.']));
    rules.push(Rule([b'.', b'.', b'.', b' ', b' ', b' ', b' ', b' ', b' ']));
    rules.push(Rule([b'.', b'.', b'.', b' ', b' ', b' ', b'.', b'.', b'.']));
    rules.push(Rule([b'.', b'.', b'.', b' ', b' ', b'.', b' ', b' ', b' ']));
    rules.push(Rule([b'.', b'.', b'.', b' ', b' ', b'.', b' ', b' ', b'.']));
    rules.push(Rule([b'.', b'.', b'.', b' ', b' ', b'.', b'.', b'.', b'.']));
    rules.push(Rule([b'.', b'.', b'.', b' ', b'.', b' ', b' ', b'.', b' ']));
    rules.push(Rule([b'.', b'.', b'.', b' ', b'.', b'.', b' ', b' ', b' ']));
    rules.push(Rule([b'.', b'.', b'.', b' ', b'.', b'.', b' ', b' ', b'.']));
    rules.push(Rule([b'.', b'.', b'.', b' ', b'.', b'.', b' ', b'.', b' ']));
    rules.push(Rule([b'.', b'.', b'.', b' ', b'.', b'.', b' ', b'.', b'.']));
    rules.push(Rule([b'.', b'.', b'.', b' ', b'.', b'.', b'.', b'.', b'.']));
    rules.push(Rule([b'.', b'.', b'.', b'.', b' ', b' ', b' ', b' ', b' ']));
    rules.push(Rule([b'.', b'.', b'.', b'.', b' ', b' ', b'.', b' ', b' ']));
    rules.push(Rule([b'.', b'.', b'.', b'.', b' ', b' ', b'.', b'.', b'.']));
    rules.push(Rule([b'.', b'.', b'.', b'.', b' ', b'.', b'.', b' ', b'.']));
    rules.push(Rule([b'.', b'.', b'.', b'.', b'.', b' ', b' ', b' ', b' ']));
    rules.push(Rule([b'.', b'.', b'.', b'.', b'.', b' ', b'.', b' ', b' ']));
    rules.push(Rule([b'.', b'.', b'.', b'.', b'.', b' ', b'.', b'.', b' ']));
    rules.push(Rule([b'.', b'.', b'.', b'.', b'.', b' ', b'.', b'.', b'.']));
    rules.push(Rule([b'.', b'.', b'.', b'.', b'.', b'.', b' ', b' ', b' ']));
    rules.push(Rule([b'.', b'.', b'.', b'.', b'.', b'.', b' ', b' ', b'.']));
    rules.push(Rule([b'.', b'.', b'.', b'.', b'.', b'.', b' ', b'.', b' ']));
    rules.push(Rule([b'.', b'.', b'.', b'.', b'.', b'.', b' ', b'.', b'.']));
    rules.push(Rule([b'.', b'.', b'.', b'.', b'.', b'.', b'.', b' ', b' ']));
    rules.push(Rule([b'.', b'.', b'.', b'.', b'.', b'.', b'.', b' ', b'.']));
    rules.push(Rule([b'.', b'.', b'.', b'.', b'.', b'.', b'.', b'.', b' ']));
    rules.push(Rule([b'.', b'.', b'.', b'.', b'.', b'.', b'.', b'.', b'.']));
    rules
}

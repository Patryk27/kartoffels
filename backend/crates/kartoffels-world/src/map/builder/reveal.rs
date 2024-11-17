use super::MapBuilder;
use crate::{Dir, Map, TileKind};
use rand::seq::SliceRandom;
use rand::{Rng, RngCore};

impl MapBuilder {
    pub async fn reveal(&mut self, map: Map, rng: &mut impl RngCore) {
        const NOT_VISITED: u8 = 0;
        const VISITED: u8 = 1;

        self.map = map;

        let mut frontier: Vec<_> = {
            let mut frontier = Vec::new();

            self.map.for_each(|pos, tile| {
                if tile.is_floor() {
                    frontier.push(pos);
                }
            });

            frontier.shuffle(rng);
            frontier.into_iter().take(3).collect()
        };

        let mut updates = 0;

        while !frontier.is_empty() {
            let idx = rng.gen_range(0..frontier.len());
            let pos = frontier.swap_remove(idx);

            if self.map.get(pos).meta[0] == VISITED {
                continue;
            }

            self.map.get_mut(pos).meta[0] = VISITED;

            for dir in Dir::all() {
                if self.map.contains(pos + dir) {
                    frontier.push(pos + dir);
                }
            }

            if let Some(tx) = &self.tx
                && updates % 32 == 0
            {
                let map = self.map.clone().map(|_, tile| {
                    if tile.meta[0] == NOT_VISITED {
                        TileKind::VOID.into()
                    } else {
                        tile
                    }
                });

                _ = tx.send(map).await;
            }

            updates += 1;
        }

        self.map.for_each_mut(|_, tile| {
            tile.meta[0] = 0;
        });
    }
}

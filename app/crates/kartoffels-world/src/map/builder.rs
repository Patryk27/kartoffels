use super::{Map, Tile};
use crate::{AbsDir, TileKind};
use glam::{IVec2, UVec2, ivec2};
use rand::seq::SliceRandom;
use rand::{Rng, RngCore};
use std::{cmp, ops};
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct MapBuilder {
    map: Map,
    label: Option<String>,
    tx: mpsc::Sender<MapUpdate>,
    changes: u32,
    frequency: u32,
}

impl MapBuilder {
    pub fn new() -> (Self, mpsc::Receiver<MapUpdate>) {
        let (tx, rx) = mpsc::channel(1);

        let this = Self {
            map: Map::default(),
            label: None,
            tx,
            changes: 0,
            frequency: 10,
        };

        (this, rx)
    }

    pub fn detached() -> Self {
        Self::new().0
    }

    pub fn set_label(&mut self, label: impl ToString) {
        self.label = Some(label.to_string());
    }

    pub fn set_frequency(&mut self, freq: f32) {
        assert!(self.map.size().length_squared() > 0);

        self.frequency =
            (1.0 / freq * self.map.size().as_vec2().length()) as u32;
    }

    pub async fn notify(&mut self) {
        if self.tx.is_closed() {
            return;
        }

        _ = self
            .tx
            .send(MapUpdate {
                map: self.map.clone(),
                label: self.label.take(),
            })
            .await;
    }

    pub fn begin(&mut self, size: UVec2) {
        self.map = Map::new(size);
    }

    pub fn commit(self) -> Map {
        self.map
    }

    pub fn with<T>(&mut self, f: impl FnOnce(&mut Map) -> T) -> T {
        f(&mut self.map)
    }

    pub async fn set(&mut self, pos: IVec2, tile: impl Into<Tile>) {
        if self.map.set(pos, tile) {
            self.tick().await;
        }
    }

    pub async fn set_if_void(&mut self, pos: IVec2, tile: impl Into<Tile>) {
        if self.get(pos).is_void() {
            self.set(pos, tile).await;
        }
    }

    // TODO duplicated with `Map::line()`
    pub async fn line(&mut self, p1: IVec2, p2: IVec2, tile: impl Into<Tile>) {
        let tile = tile.into();

        if p1.x == p2.x {
            let [y1, y2] = cmp::minmax(p1.y, p2.y);

            for y in y1..=y2 {
                self.set(ivec2(p1.x, y), tile).await;
            }
        } else if p1.y == p2.y {
            let [x1, x2] = cmp::minmax(p1.x, p2.x);

            for x in x1..=x2 {
                self.set(ivec2(x, p1.y), tile).await;
            }
        } else {
            unimplemented!();
        }
    }

    pub async fn reveal(&mut self, rng: &mut impl RngCore, map: Map) {
        const NOT_VISITED: u8 = 0;
        const VISITED: u8 = 1;

        self.map = map;

        // If there's no listener for our changes, exit early so that we don't
        // touch `rng`.
        //
        // Comes handy for tests, because otherwise any change to the algorithm
        // below could affect the PRNG and thus require reblessing the tests.
        if self.tx.is_closed() {
            return;
        }

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

        let mut changes = 0;

        while !frontier.is_empty() {
            let idx = rng.gen_range(0..frontier.len());
            let pos = frontier.swap_remove(idx);

            if self.map.get(pos).meta[0] == VISITED {
                continue;
            }

            self.map.get_mut(pos).meta[0] = VISITED;

            for dir in AbsDir::all() {
                if self.map.contains(pos + dir) {
                    frontier.push(pos + dir);
                }
            }

            if changes % 32 == 0 {
                let map = self.map.clone().map(|_, tile| {
                    if tile.meta[0] == NOT_VISITED {
                        TileKind::VOID.into()
                    } else {
                        tile
                    }
                });

                _ = self.tx.send(MapUpdate { map, label: None }).await;
            }

            changes += 1;
        }

        self.map.for_each_mut(|_, tile| {
            tile.meta[0] = 0;
        });
    }

    async fn tick(&mut self) {
        if self.changes % self.frequency == 0 {
            self.notify().await;
        }

        self.changes += 1;
    }
}

impl ops::Deref for MapBuilder {
    type Target = Map;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

#[derive(Clone, Debug)]
pub struct MapUpdate {
    pub map: Map,
    pub label: Option<String>,
}

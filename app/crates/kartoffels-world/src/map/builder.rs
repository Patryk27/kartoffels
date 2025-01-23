mod reveal;

use super::{Map, Tile};
use glam::{ivec2, IVec2, UVec2};
use std::cmp;
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct MapBuilder {
    map: Map,
    tx: mpsc::Sender<Map>,
    updates: u32,
}

impl MapBuilder {
    pub fn new() -> (Self, mpsc::Receiver<Map>) {
        let (tx, rx) = mpsc::channel(1);

        let this = Self {
            map: Default::default(),
            tx,
            updates: 0,
        };

        (this, rx)
    }

    pub fn init(&mut self, size: UVec2) {
        self.map = Map::new(size);
    }

    pub fn update<T>(&mut self, f: impl FnOnce(&mut Map) -> T) -> T {
        f(&mut self.map)
    }

    pub fn get(&self, pos: IVec2) -> Tile {
        self.map.get(pos)
    }

    pub async fn set(&mut self, pos: IVec2, tile: impl Into<Tile>) {
        self.map.set(pos, tile);
        self.notify().await;
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

    pub async fn notify(&mut self) {
        if self.updates % 10 == 0 {
            _ = self.tx.send(self.map.clone()).await;
        }

        self.updates += 1;
    }

    pub fn finish(self) -> Map {
        self.map
    }
}

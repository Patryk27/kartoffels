use super::{Map, MapBuilder};
use crate::{spec, Dir, TileKind};
use ahash::AHashSet;
use anyhow::{anyhow, Context, Error, Result};
use glam::{ivec2, uvec2, IVec2, UVec2};
use rand::seq::SliceRandom;
use rand::{Rng, RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::collections::VecDeque;
use std::iter;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaveTheme {
    size: UVec2,
}

impl CaveTheme {
    pub fn new(size: UVec2) -> Self {
        Self { size }
    }

    pub async fn build(
        &self,
        rng: &mut impl RngCore,
        mut map: MapBuilder,
    ) -> Result<Map> {
        let mut rng = {
            let mut idx = 0;

            loop {
                map.set_status(format!("evaluating-subseed:{idx}"));
                map.notify().await;

                idx += 1;

                // ---

                let sample_rng = ChaCha8Rng::from_seed(rng.gen());

                let sample_map = self
                    .build_once(&mut sample_rng.clone(), MapBuilder::detached())
                    .await;

                if self.is_good_enough(&sample_map) {
                    break sample_rng;
                }
            }
        };

        Ok(self.build_once(&mut rng, map).await)
    }

    async fn build_once(
        &self,
        rng: &mut impl RngCore,
        mut map: MapBuilder,
    ) -> Map {
        map.begin(self.size);

        self.rand_pass(rng, &mut map).await;

        for idx in 0..5 {
            self.smooth_pass(rng, &mut map, idx).await;
        }

        self.fill_pass(rng, &mut map).await;

        map.commit()
    }

    async fn rand_pass(&self, rng: &mut impl RngCore, map: &mut MapBuilder) {
        map.set_status("rand-pass");
        map.set_notify_every(100);

        for pos in self.sample_points(rng) {
            let tile = if rng.gen_bool(0.45) {
                TileKind::WALL
            } else {
                TileKind::FLOOR
            };

            map.set(pos, tile).await;
        }
    }

    async fn smooth_pass(
        &self,
        rng: &mut impl RngCore,
        map: &mut MapBuilder,
        idx: u32,
    ) {
        map.set_status(format!("smooth-pass:{idx}"));
        map.set_notify_every(15 - idx);

        let prev_map = (*map).clone();

        for pos in self.sample_points(rng) {
            let mut r1 = 0;
            let mut r2 = 0;

            for dy in -2..=2 {
                for dx in -2..=2 {
                    if prev_map.get(pos + ivec2(dx, dy)).is_wall() {
                        if dx.abs() <= 1 && dy.abs() <= 1 {
                            r1 += 1;
                        }

                        r2 += 1;
                    }
                }
            }

            let is_border = pos.x == 0
                || pos.y == 0
                || pos.x == (self.size.x as i32 - 1)
                || pos.y == (self.size.y as i32 - 1);

            if is_border || (r1 >= 5 && r2 <= 19) {
                map.set(pos, TileKind::WALL).await;
            } else if r2 <= 11 {
                map.set(pos, TileKind::FLOOR).await;
            }
        }
    }

    async fn fill_pass(&self, rng: &mut impl RngCore, map: &mut MapBuilder) {
        map.set_status("fill-pass");

        let mut caves = Vec::new();

        let mut points: AHashSet<_> = self
            .sample_points(rng)
            .filter(|pos| map.get(*pos).is_floor())
            .collect();

        while let Some(pos) = points.iter().next().copied() {
            points.remove(&pos);

            let mut cave = Vec::new();
            let mut stack = VecDeque::from_iter(iter::once(pos));

            while let Some(pos) = stack.pop_front() {
                cave.push(pos);

                for dir in Dir::all() {
                    let pos = pos + dir;

                    if map.get(pos).is_floor() && points.remove(&pos) {
                        stack.push_back(pos);
                    }
                }
            }

            caves.push(cave);
        }

        caves.sort_by_key(|cave| Reverse(cave.len()));

        // ---

        map.set_notify_every(10);

        for cave in caves.iter().skip(1) {
            for pos in cave {
                map.set(*pos, TileKind::WALL).await;
            }
        }
    }

    fn is_good_enough(&self, map: &Map) -> bool {
        let mut floors = 0;
        let mut walls = 0;

        map.for_each(|_, tile| {
            if tile.is_floor() {
                floors += 1;
            } else {
                walls += 1;
            }
        });

        floors >= walls
    }

    fn sample_points(
        &self,
        rng: &mut impl RngCore,
    ) -> impl Iterator<Item = IVec2> {
        let mut poss = Vec::new();

        for y in 0..self.size.y {
            for x in 0..self.size.x {
                poss.push(ivec2(x as i32, y as i32));
            }
        }

        poss.shuffle(rng);
        poss.into_iter()
    }
}

impl FromStr for CaveTheme {
    type Err = Error;

    fn from_str(spec: &str) -> Result<Self> {
        let mut width = None;
        let mut height = None;

        for entry in spec::entries(spec) {
            let entry = entry?;

            match entry.key {
                "width" => {
                    width = Some(entry.value()?);
                }
                "height" => {
                    height = Some(entry.value()?);
                }
                key => {
                    return Err(anyhow!("unknown key: {key}"));
                }
            }
        }

        let width = width.context("missing key: width")?;
        let height = height.context("missing key: height")?;

        Ok(Self::new(uvec2(width, height)))
    }
}

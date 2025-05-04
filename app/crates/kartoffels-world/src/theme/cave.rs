use crate::*;

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
                map.set_label(format!("evaluating-subseed:{idx}"));
                map.notify().await;

                idx += 1;

                // ---

                let sample_rng = ChaCha8Rng::from_seed(rng.gen());

                let sample_map = self
                    .build_ex(&mut sample_rng.clone(), MapBuilder::detached())
                    .await;

                if self.is_good_enough(&sample_map) {
                    break sample_rng;
                }
            }
        };

        Ok(self.build_ex(&mut rng, map).await)
    }

    async fn build_ex(
        &self,
        rng: &mut impl RngCore,
        mut map: MapBuilder,
    ) -> Map {
        map.begin(self.size);

        self.rand_pass(rng, &mut map).await;

        for idx in 0..2 {
            self.smooth_pass(rng, &mut map, idx).await;
        }

        self.fill_pass(rng, &mut map).await;

        map.commit()
    }

    async fn rand_pass(&self, rng: &mut impl RngCore, map: &mut MapBuilder) {
        map.set_label("rand-pass");
        map.set_frequency(1.0);

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
        map.set_label(format!("smooth-pass:{idx}"));
        map.set_frequency(4.0);

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
        map.set_label("fill-pass");
        map.set_frequency(12.0);

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

            cave.shuffle(rng);
            caves.push(cave);
        }

        caves.sort_by_key(|cave| Reverse(cave.len()));

        // ---

        let mut points: Vec<_> = caves.into_iter().skip(1).flatten().collect();

        points.shuffle(rng);

        for pos in points {
            map.set(pos, TileKind::WALL).await;
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

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::FutureExt;
    use kartoffels_utils::Asserter;

    #[test]
    fn build() {
        let mut rng = ChaCha8Rng::from_seed(Default::default());

        let map = CaveTheme::new(uvec2(64, 32))
            .build(&mut rng, MapBuilder::detached())
            .now_or_never()
            .unwrap()
            .unwrap();

        Asserter::new("src/theme/cave/tests")
            .assert("build.txt", map.to_string());
    }
}

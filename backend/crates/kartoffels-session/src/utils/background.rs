use glam::{ivec2, uvec2, IVec2, UVec2};
use kartoffels_ui::{theme, Term, Ui};
use kartoffels_world::prelude::{Dir, DungeonTheme, Map, Tile, TileBase};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::sync::{Arc, LazyLock};
use std::thread;
use std::time::Duration;
use tokio::sync::watch;

pub struct Background {
    stream: watch::Receiver<Arc<Map>>,
    camera: IVec2,
}

impl Background {
    const MAP_SIZE: UVec2 = uvec2(256, 256);

    pub fn new(term: &Term) -> Self {
        let stream = STREAM.subscribe();

        // Sample the camera position, so that every time the user opens the
        // menu, we show them a different piece of map.
        //
        // Crucially, avoid sampling points which would cause the viewport to
        // land outside the map (hence we need to know the terminal size here).
        let camera = {
            let mut rng = rand::thread_rng();

            let min = uvec2(0, 0);
            let max = Self::MAP_SIZE - term.size();

            uvec2(rng.gen_range(min.x..=max.x), rng.gen_range(min.y..=max.y))
                .as_ivec2()
        };

        Self { stream, camera }
    }

    pub fn render<T>(&mut self, ui: &mut Ui<T>) {
        let map = self.stream.borrow().clone();

        for x in 0..ui.area().width {
            for y in 0..ui.area().height {
                let pos = self.camera + ivec2(x as i32, y as i32);

                ui.buf()[(x, y)].reset();

                ui.buf()[(x, y)]
                    .set_bg(theme::BG)
                    .set_fg(theme::DARK_GRAY)
                    .set_char(map.get(pos).base as char);
            }
        }
    }
}

static STREAM: LazyLock<watch::Sender<Arc<Map>>> = LazyLock::new(|| {
    let tx = watch::Sender::new(Default::default());

    thread::spawn({
        let tx = tx.clone();

        move || {
            refresh(tx);
        }
    });

    tx
});

fn refresh(tx: watch::Sender<Arc<Map>>) {
    let mut rng = ChaCha8Rng::from_seed(Default::default());

    let mut map = DungeonTheme::new(Background::MAP_SIZE)
        .create_map(&mut rng)
        .unwrap();

    for y in 0..map.size().y {
        for x in 0..map.size().x {
            let point = ivec2(x as i32, y as i32);

            if map.get(point).base == TileBase::FLOOR && rng.gen_bool(0.05) {
                map.set(point, Tile::new(TileBase::BOT));
            }
        }
    }

    let mut frame = 0;

    loop {
        frame += 1;

        for y in 0..map.size().y {
            for x in 0..map.size().x {
                let src = ivec2(x as i32, y as i32);
                let src_tile = map.get(src);

                if src_tile.base == TileBase::BOT
                    && src_tile.meta[0] != frame
                    && rng.gen_bool(0.33)
                {
                    let dst = src + rng.gen::<Dir>().as_vec();

                    if map.get(dst).base == TileBase::FLOOR {
                        map.set(src, Tile::new(TileBase::FLOOR));

                        map.set(
                            dst,
                            Tile {
                                base: TileBase::BOT,
                                meta: [frame, 0, 0],
                            },
                        );
                    }
                }
            }
        }

        tx.send_replace(Arc::new(map.clone()));

        thread::sleep(Duration::from_millis(125));
    }
}

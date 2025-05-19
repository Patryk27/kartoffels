use crate::{theme, Frame, Ui, UiWidget};
use futures::FutureExt;
use glam::{ivec2, uvec2, IVec2, UVec2};
use kartoffels_store::Store;
use kartoffels_world::prelude as w;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::sync::{Arc, LazyLock};
use std::thread;
use std::time::Duration;
use tokio::sync::watch;

#[derive(Debug)]
pub struct BgMap {
    stream: watch::Receiver<Arc<w::Map>>,
    camera: IVec2,
}

impl BgMap {
    const MAP_SIZE: UVec2 = uvec2(256, 256);

    pub fn new(store: &Store, frame: &Frame) -> Self {
        let stream = if store.testing() {
            watch::channel(Default::default()).1
        } else {
            STREAM.subscribe()
        };

        // Sample the camera position, so that every time the user opens the
        // menu, we show them a different piece of map.
        //
        // Crucially, avoid sampling points which would cause the viewport to
        // land outside the map (hence we need to know the terminal size here).
        let camera = {
            let mut rng = rand::thread_rng();

            let min = uvec2(0, 0);
            let max = Self::MAP_SIZE - frame.size();

            uvec2(rng.gen_range(min.x..=max.x), rng.gen_range(min.y..=max.y))
                .as_ivec2()
        };

        Self { stream, camera }
    }

    pub fn init() {
        // Borrow the stream to force it to initialize - this way the first
        // person to connect won't get a black background, since it'll have been
        // initialized by then.
        STREAM.borrow();
    }
}

impl<T> UiWidget<T> for &BgMap {
    fn render(self, ui: &mut Ui<T>) -> Self::Response {
        let map = self.stream.borrow().clone();

        for x in 0..ui.area.width {
            for y in 0..ui.area.height {
                let pos = self.camera + ivec2(x as i32, y as i32);

                ui.buf[(x, y)].reset();

                ui.buf[(x, y)]
                    .set_bg(theme::BG)
                    .set_fg(theme::DARK_GRAY)
                    .set_char(map.get(pos).kind as char);
            }
        }
    }
}

static STREAM: LazyLock<watch::Sender<Arc<w::Map>>> = LazyLock::new(|| {
    let tx = watch::Sender::new(Default::default());

    thread::spawn({
        let tx = tx.clone();

        move || {
            refresh(tx);
        }
    });

    tx
});

fn refresh(tx: watch::Sender<Arc<w::Map>>) {
    let mut rng = ChaCha8Rng::from_seed(Default::default());

    let mut map = w::CaveTheme::new(BgMap::MAP_SIZE)
        .build(&mut rng, w::MapBuilder::detached())
        .now_or_never()
        .unwrap()
        .unwrap();

    map.for_each_mut(|_, tile| {
        if tile.is_floor() && rng.gen_bool(0.05) {
            *tile = w::TileKind::BOT.into();
        }
    });

    let mut frame = 0;

    loop {
        frame += 1;

        for y in 0..map.size().y {
            for x in 0..map.size().x {
                let src = ivec2(x as i32, y as i32);
                let src_tile = map.get(src);

                if src_tile.kind == w::TileKind::BOT
                    && src_tile.meta[0] != frame
                    && rng.gen_bool(0.33)
                {
                    let dst = src + rng.gen::<w::AbsDir>();

                    if map.get(dst).kind == w::TileKind::FLOOR {
                        map.set(src, w::TileKind::FLOOR);

                        map.set(
                            dst,
                            w::Tile {
                                kind: w::TileKind::BOT,
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

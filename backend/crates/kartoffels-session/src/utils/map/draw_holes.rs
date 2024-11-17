use kartoffels_world::prelude::{Map, TileKind};
use rand::RngCore;
use tokio::sync::mpsc;
use tokio::task;

pub async fn draw_holes(
    map: &mut Map,
    rng: &mut impl RngCore,
    progress: &mpsc::Sender<Map>,
    mut how_many: u32,
) {
    while how_many > 0 {
        let pos = map.sample_pos(rng);

        if map.get(pos).is_wall() {
            map.set(pos, TileKind::FLOOR);
            _ = progress.send(map.clone()).await;
            how_many -= 1;
        }

        task::yield_now().await;
    }
}

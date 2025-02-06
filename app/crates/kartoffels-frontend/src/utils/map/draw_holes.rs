use kartoffels_world::prelude::{MapBuilder, TileKind};
use rand::RngCore;
use tokio::task;

pub async fn draw_holes(
    rng: &mut impl RngCore,
    map: &mut MapBuilder,
    mut how_many: u32,
) {
    while how_many > 0 {
        let pos = map.with(|map| map.sample_pos(rng));

        if map.get(pos).is_wall() {
            map.set(pos, TileKind::FLOOR).await;
            how_many -= 1;
        }

        task::yield_now().await;
    }
}

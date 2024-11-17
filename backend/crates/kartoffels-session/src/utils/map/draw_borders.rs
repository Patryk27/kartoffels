use glam::{ivec2, UVec2};
use kartoffels_ui::theme;
use kartoffels_world::prelude::{Map, TileKind};
use tokio::sync::mpsc;
use tokio::time;

pub async fn draw_borders(
    map: &mut Map,
    size: UVec2,
    progress: &mpsc::Sender<Map>,
) {
    let size = size.as_ivec2();

    // ---

    time::sleep(6 * theme::FRAME_TIME).await;

    map.line(ivec2(0, 0), ivec2(size.x - 1, 0), TileKind::WALL_H);

    _ = progress.send(map.clone()).await;

    // ---

    time::sleep(6 * theme::FRAME_TIME).await;

    map.line(ivec2(0, 1), ivec2(0, size.y - 2), TileKind::WALL_V);

    _ = progress.send(map.clone()).await;

    // ---

    time::sleep(6 * theme::FRAME_TIME).await;

    map.line(
        ivec2(0, size.y - 1),
        ivec2(size.x - 1, size.y - 1),
        TileKind::WALL_H,
    );

    _ = progress.send(map.clone()).await;

    // ---

    time::sleep(6 * theme::FRAME_TIME).await;

    map.line(
        ivec2(size.x - 1, 1),
        ivec2(size.x - 1, size.y - 2),
        TileKind::WALL_V,
    );

    _ = progress.send(map.clone()).await;
}

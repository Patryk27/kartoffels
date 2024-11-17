use glam::{ivec2, UVec2};
use kartoffels_world::prelude::{MapBuilder, TileKind};

pub async fn draw_borders(map: &mut MapBuilder, size: UVec2) {
    let size = size.as_ivec2();

    map.line(ivec2(0, 0), ivec2(size.x - 1, 0), TileKind::WALL_H)
        .await;

    map.line(ivec2(0, 1), ivec2(0, size.y - 2), TileKind::WALL_V)
        .await;

    map.line(
        ivec2(0, size.y - 1),
        ivec2(size.x - 1, size.y - 1),
        TileKind::WALL_H,
    )
    .await;

    map.line(
        ivec2(size.x - 1, 1),
        ivec2(size.x - 1, size.y - 2),
        TileKind::WALL_V,
    )
    .await;
}

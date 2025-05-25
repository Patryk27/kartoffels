use glam::{UVec2, ivec2};
use kartoffels_world::prelude as w;

pub async fn draw_borders(map: &mut w::MapBuilder, size: UVec2) {
    let size = size.as_ivec2();

    map.line(ivec2(0, 0), ivec2(size.x - 1, 0), w::TileKind::WALL_H)
        .await;

    map.line(ivec2(0, 1), ivec2(0, size.y - 2), w::TileKind::WALL_V)
        .await;

    map.line(
        ivec2(0, size.y - 1),
        ivec2(size.x - 1, size.y - 1),
        w::TileKind::WALL_H,
    )
    .await;

    map.line(
        ivec2(size.x - 1, 1),
        ivec2(size.x - 1, size.y - 2),
        w::TileKind::WALL_V,
    )
    .await;
}

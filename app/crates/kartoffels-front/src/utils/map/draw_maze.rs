use glam::{ivec2, IVec2, UVec2};
use kartoffels_world::prelude::{AbsDir, MapBuilder, TileKind};
use rand::{Rng, RngCore};

/// Creates an acyclic maze using recursive backtracking algorithm, a'la
/// https://weblog.jamisbuck.org/2010/12/27/maze-generation-recursive-backtracking
pub async fn draw_maze(
    rng: &mut impl RngCore,
    map: &mut MapBuilder,
    area: UVec2,
    head: IVec2,
) {
    const NOT_VISITED: u8 = 0;
    const VISITED: u8 = 1;

    let mut frontier = Vec::new();

    for dir in AbsDir::shuffled(rng) {
        frontier.push((head, dir));
    }

    map.with(|map| {
        map.get_mut(head).meta[0] = VISITED;
    });

    while !frontier.is_empty() {
        let idx = rng.gen_range(0..frontier.len());
        let (src, dir) = frontier.swap_remove(idx);
        let mid = src + dir;
        let dst = mid + dir;

        if map.get(src).is_void() {
            map.with(|map| {
                map.get_mut(src).kind = TileKind::FLOOR;
            });

            map.set_if_void(src - ivec2(1, 0), TileKind::WALL_V).await;
            map.set_if_void(src + ivec2(1, 0), TileKind::WALL_V).await;
            map.set_if_void(src - ivec2(0, 1), TileKind::WALL_H).await;
            map.set_if_void(src + ivec2(0, 1), TileKind::WALL_H).await;
        }

        if dst.x >= 0
            && dst.y >= 0
            && dst.x < area.x as i32
            && dst.y < area.y as i32
            && map.get(dst).meta[0] == NOT_VISITED
        {
            map.with(|map| {
                map.get_mut(dst).meta[0] = VISITED;
            });

            map.set(mid, TileKind::FLOOR).await;

            match dir {
                AbsDir::N | AbsDir::S => {
                    map.set(mid - ivec2(1, 0), TileKind::WALL_V).await;
                    map.set(mid + ivec2(1, 0), TileKind::WALL_V).await;
                }

                AbsDir::E | AbsDir::W => {
                    map.set(mid - ivec2(0, 1), TileKind::WALL_H).await;
                    map.set(mid + ivec2(0, 1), TileKind::WALL_H).await;
                }
            }

            for dir in AbsDir::shuffled(rng) {
                frontier.push((dst, dir));
            }
        }
    }

    map.with(|map| {
        map.for_each_mut(|_, tile| {
            tile.meta[0] = 0;
        });
    });
}

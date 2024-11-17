use glam::{ivec2, IVec2, UVec2};
use kartoffels_world::prelude::{Dir, Map, TileKind};
use rand::{Rng, RngCore};
use tokio::sync::mpsc;

/// Creates an acyclic maze using recursive backtracking algorithm, a'la
/// https://weblog.jamisbuck.org/2010/12/27/maze-generation-recursive-backtracking
pub async fn draw_maze(
    map: &mut Map,
    rng: &mut impl RngCore,
    progress: &mpsc::Sender<Map>,
    area: UVec2,
    head: IVec2,
) {
    const NOT_VISITED: u8 = 0;
    const VISITED: u8 = 1;

    let mut nth = 0;
    let mut frontier = Vec::new();

    for dir in Dir::shuffled(rng) {
        frontier.push((head, dir));
    }

    map.get_mut(head).meta[0] = VISITED;

    while !frontier.is_empty() {
        let idx = rng.gen_range(0..frontier.len());
        let (src_pos, dir) = frontier.swap_remove(idx);
        let mid_pos = src_pos + dir;
        let dst_pos = mid_pos + dir;

        if map.get(src_pos).is_void() {
            map.get_mut(src_pos).kind = TileKind::FLOOR;
            map.set_if_void(src_pos - ivec2(1, 0), TileKind::WALL_V);
            map.set_if_void(src_pos + ivec2(1, 0), TileKind::WALL_V);
            map.set_if_void(src_pos - ivec2(0, 1), TileKind::WALL_H);
            map.set_if_void(src_pos + ivec2(0, 1), TileKind::WALL_H);
        }

        if dst_pos.x >= 0
            && dst_pos.y >= 0
            && dst_pos.x < area.x as i32
            && dst_pos.y < area.y as i32
            && map.get(dst_pos).meta[0] == NOT_VISITED
        {
            map.get_mut(dst_pos).meta[0] = VISITED;
            map.set(mid_pos, TileKind::FLOOR);

            match dir {
                Dir::N | Dir::S => {
                    map.set(mid_pos - ivec2(1, 0), TileKind::WALL_V);
                    map.set(mid_pos + ivec2(1, 0), TileKind::WALL_V);
                }

                Dir::E | Dir::W => {
                    map.set(mid_pos - ivec2(0, 1), TileKind::WALL_H);
                    map.set(mid_pos + ivec2(0, 1), TileKind::WALL_H);
                }
            }

            for dir in Dir::shuffled(rng) {
                frontier.push((dst_pos, dir));
            }

            if nth % 3 == 0 {
                _ = progress.send(map.clone()).await;
            }

            nth += 1;
        }
    }

    map.for_each_mut(|_, tile| {
        tile.meta[0] = 0;
    });
}

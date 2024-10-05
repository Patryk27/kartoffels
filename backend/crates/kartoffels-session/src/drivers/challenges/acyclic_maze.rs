use super::prelude::*;
use anyhow::Error;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use tokio::sync::mpsc;
use tokio::try_join;

pub static CHALLENGE: Challenge = Challenge {
    name: "acyclic-maze",
    desc: "your bot got lost and it's visibly distressed, help it escape!",
    run,
};

fn run(store: &Store, game: DrivenGame) -> BoxFuture<'_, Result<()>> {
    Box::pin(async move {
        game.set_perms(Permissions::PENDING).await?;
        game.set_status(Some("BUILDING WORLD".into())).await?;

        let world = store.create_world(Config {
            clock: Default::default(),
            mode: Mode::Deathmatch(DeathmatchMode::default()),
            name: "sandbox".into(),
            path: Default::default(),
            policy: Policy {
                auto_respawn: true,
                max_alive_bots: 1,
                max_queued_bots: 1,
            },
            rng: None,
            theme: None,
        })?;

        game.join(world.clone()).await?;

        let (tx, mut rx) = mpsc::channel(1);

        let map = async move {
            let rng = ChaCha8Rng::from_seed(rand::thread_rng().gen());
            let map = create_map(rng, tx).await;

            Ok(map)
        };

        let progress = async {
            while let Some(map) = rx.recv().await {
                world.set_map(map).await?;

                time::sleep(Duration::from_millis(16)).await;
            }

            Ok(())
        };

        let (map, _) = try_join!(map, progress).map_err(|err: Error| err)?;

        world.set_map(map).await?;

        game.set_perms(Permissions::CHALLENGE).await?;
        game.set_status(None).await?;

        future::pending::<()>().await;

        Ok(())
    })
}

async fn create_map(mut rng: impl RngCore, tx: mpsc::Sender<Map>) -> Map {
    let cells = uvec2(18, 10);
    let size = 2 * cells + 1;

    let mut map = Map::new(size);

    // ---

    map.line(ivec2(1, 0), ivec2(size.x as i32 - 2, 0), TileBase::WALL_H);
    map.line(ivec2(0, 1), ivec2(0, size.y as i32 - 2), TileBase::WALL_V);

    map.line(
        ivec2(1, size.y as i32 - 1),
        ivec2(size.x as i32 - 2, size.y as i32 - 1),
        TileBase::WALL_H,
    );

    map.line(
        ivec2(size.x as i32 - 1, 1),
        ivec2(size.x as i32 - 1, size.y as i32 - 2),
        TileBase::WALL_V,
    );

    // ---

    let mut nth = 0;
    let mut frontier = Vec::new();

    for dir in Dir::all() {
        frontier.push((ivec2(1, 1), dir));
    }

    while !frontier.is_empty() {
        let idx = rng.gen_range(0..frontier.len());
        let (src_pos, dir) = frontier.swap_remove(idx);
        let mid_pos = src_pos + dir;
        let dst_pos = mid_pos + dir;

        map.set_if_void(src_pos, TileBase::FLOOR);
        map.set_if_void(src_pos - ivec2(1, 0), TileBase::WALL_V);
        map.set_if_void(src_pos + ivec2(1, 0), TileBase::WALL_V);
        map.set_if_void(src_pos - ivec2(0, 1), TileBase::WALL_H);
        map.set_if_void(src_pos + ivec2(0, 1), TileBase::WALL_H);

        if map.contains(dst_pos) && map.get(dst_pos).meta[0] == 0 {
            map.get_mut(dst_pos).meta[0] = 1;
            map.set(mid_pos, TileBase::FLOOR);

            match dir {
                Dir::N | Dir::S => {
                    map.set(mid_pos - ivec2(1, 0), TileBase::WALL_V);
                    map.set(mid_pos + ivec2(1, 0), TileBase::WALL_V);
                }

                Dir::E | Dir::W => {
                    map.set(mid_pos - ivec2(0, 1), TileBase::WALL_H);
                    map.set(mid_pos + ivec2(0, 1), TileBase::WALL_H);
                }
            }

            for dir in Dir::all() {
                frontier.push((dst_pos, dir));
            }

            if nth % 4 == 0 {
                _ = tx.send(map.clone()).await;
            }

            nth += 1;
        }
    }

    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor;
    use kartoffels_utils::Asserter;
    use std::path::Path;

    #[test]
    fn map() {
        let dir = Path::new("src")
            .join("drivers")
            .join("challenges")
            .join("acyclic_maze")
            .join("tests")
            .join("map");

        let mut rng = ChaCha8Rng::from_seed(Default::default());
        let (tx, _) = mpsc::channel(1);
        let map = executor::block_on(create_map(&mut rng, tx));

        Asserter::new(dir).assert("expected.txt", map.to_string());
    }
}

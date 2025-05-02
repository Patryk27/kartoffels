use crate::views::game::GameCtrl;
use anyhow::{Error, Result};
use kartoffels_store::Store;
use kartoffels_ui::theme;
use kartoffels_world::prelude::{Handle, Map, MapBuilder};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::future::Future;
use tokio::{time, try_join};
use tracing::debug;

pub async fn build<BuildMapFn, BuildMapFut>(
    store: &Store,
    game: &GameCtrl,
    world: &Handle,
    build: BuildMapFn,
) -> Result<()>
where
    BuildMapFn: FnOnce(ChaCha8Rng, MapBuilder) -> BuildMapFut,
    BuildMapFut: Future<Output = Result<Map>>,
{
    let rng = {
        let seed = if store.testing() {
            Default::default()
        } else {
            rand::random()
        };

        debug!(?seed, "building world");

        ChaCha8Rng::from_seed(seed)
    };

    let (map, mut rx) = MapBuilder::new();
    let map = (build)(rng, map);

    if store.testing() {
        drop(rx);

        world.set_map(map.await?).await?;
    } else {
        let progress = async {
            while let Some(msg) = rx.recv().await {
                if let Some(label) = msg.label {
                    game.set_label(Some(format!("building:{label}"))).await?;
                }

                world.set_map(msg.map).await?;

                time::sleep(theme::FRAME_TIME).await;
            }

            Ok(())
        };

        let (map, _) = try_join!(map, progress).map_err(|err: Error| err)?;

        world.set_map(map).await?;

        time::sleep(2 * theme::FRAME_TIME).await;
    }

    Ok(())
}

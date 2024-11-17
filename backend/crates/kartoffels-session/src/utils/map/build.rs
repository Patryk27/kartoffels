use anyhow::{Error, Result};
use kartoffels_store::Store;
use kartoffels_ui::theme;
use kartoffels_world::prelude::{Handle, Map, MapBuilder};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::future::Future;
use tokio::{time, try_join};
use tracing::debug;

pub async fn build<BuildMapFn, BuildMapFut>(
    store: &Store,
    world: &Handle,
    build: BuildMapFn,
) -> Result<()>
where
    BuildMapFn: FnOnce(MapBuilder, ChaCha8Rng) -> BuildMapFut,
    BuildMapFut: Future<Output = Result<Map>>,
{
    let rng = {
        let seed = if store.testing() {
            Default::default()
        } else {
            rand::thread_rng().gen()
        };

        debug!(?seed, "building world");

        ChaCha8Rng::from_seed(seed)
    };

    let (map, mut rx) = MapBuilder::new(!store.testing());
    let map = (build)(map, rng);

    let progress = async {
        while let Some(map) = rx.recv().await {
            if store.testing() {
                continue;
            }

            world.set_map(map).await?;
            time::sleep(theme::FRAME_TIME).await;
        }

        Ok(())
    };

    let (map, _) = try_join!(map, progress).map_err(|err: Error| err)?;

    world.set_map(map).await?;

    time::sleep(2 * theme::FRAME_TIME).await;

    Ok(())
}

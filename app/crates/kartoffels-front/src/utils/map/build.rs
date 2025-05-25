use crate::theme;
use crate::views::game::GameCtrl;
use anyhow::{Error, Result};
use kartoffels_store::Store;
use kartoffels_world::prelude as w;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use tokio::{time, try_join};
use tracing::debug;

pub async fn build(
    store: &Store,
    game: &GameCtrl,
    world: &w::Handle,
    build: impl AsyncFnOnce(ChaCha8Rng, w::MapBuilder) -> Result<w::Map>,
) -> Result<()> {
    let rng = {
        let seed = if store.testing() {
            Default::default()
        } else {
            rand::random()
        };

        debug!(?seed, "building world");

        ChaCha8Rng::from_seed(seed)
    };

    let (map, mut rx) = w::MapBuilder::new();
    let map = (build)(rng, map);

    if store.testing() {
        drop(rx);
        world.set_map(map.await?).await?;

        return Ok(());
    }

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

    Ok(())
}

use anyhow::{Error, Result};
use kartoffels_store::Store;
use kartoffels_ui::theme;
use kartoffels_world::prelude::{Handle, Map};
use rand::Rng;
use std::future::Future;
use tokio::sync::mpsc;
use tokio::{time, try_join};

type RngSeed = [u8; 32];

pub async fn reveal<CreateMapFn, CreateMapFut>(
    store: &Store,
    world: &Handle,
    create_map: CreateMapFn,
) -> Result<()>
where
    CreateMapFn: FnOnce(RngSeed, mpsc::Sender<Map>) -> CreateMapFut,
    CreateMapFut: Future<Output = Result<Map>>,
{
    let seed = if store.testing() {
        Default::default()
    } else {
        rand::thread_rng().gen()
    };

    let (tx, mut rx) = mpsc::channel(1);
    let map = (create_map)(seed, tx);

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

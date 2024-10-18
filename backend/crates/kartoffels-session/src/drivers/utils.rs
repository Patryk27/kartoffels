use anyhow::{Error, Result};
use kartoffels_store::Store;
use kartoffels_ui::theme;
use kartoffels_world::prelude::{Handle, Map};
use std::future::Future;
use tokio::sync::mpsc;
use tokio::{time, try_join};

pub async fn create_map<CreateMapFn, CreateMapFut>(
    store: &Store,
    world: &Handle,
    create_map: CreateMapFn,
) -> Result<()>
where
    CreateMapFn: FnOnce(mpsc::Sender<Map>) -> CreateMapFut,
    CreateMapFut: Future<Output = Result<Map>>,
{
    let (tx, mut rx) = mpsc::channel(1);
    let map = create_map(tx);

    let progress = async {
        while let Some(map) = rx.recv().await {
            if store.is_testing() {
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

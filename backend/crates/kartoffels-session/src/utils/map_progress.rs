use anyhow::{Error, Result};
use kartoffels_store::Store;
use kartoffels_ui::theme;
use kartoffels_world::prelude::{Handle, Map};
use std::future::Future;
use tokio::sync::mpsc;
use tokio::{time, try_join};

pub struct MapProgress<Fn> {
    fun: Fn,
}

impl<Fn, Fut> MapProgress<Fn>
where
    Fn: FnOnce(mpsc::Sender<Map>) -> Fut,
    Fut: Future<Output = Result<Map>>,
{
    pub fn new(fun: Fn) -> Self {
        Self { fun }
    }

    pub async fn run(self, store: &Store, world: &Handle) -> Result<()> {
        let (tx, mut rx) = mpsc::channel(1);
        let map = (self.fun)(tx);

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
}

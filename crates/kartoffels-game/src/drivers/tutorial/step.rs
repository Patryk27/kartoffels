use crate::DrivenGame;
use anyhow::Result;
use kartoffels_store::Store;
use kartoffels_ui::{theme, Dialog};
use kartoffels_world::prelude::Handle;
use tokio::sync::oneshot;
use tokio::time;

#[derive(Debug)]
pub struct StepCtxt<'a> {
    pub store: &'a Store,
    pub game: DrivenGame,
    pub world: Option<Handle>,
}

impl StepCtxt<'_> {
    pub async fn dialog<T>(&self, dialog: &'static Dialog<T>) -> Result<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        let (tx, rx) = oneshot::channel();
        let mut tx = Some(tx);

        self.game
            .open_dialog(move |ui| {
                if let Some(resp) = dialog.render(ui) {
                    if let Some(tx) = tx.take() {
                        _ = tx.send(resp);
                    }
                }
            })
            .await?;

        let response = rx.await?;

        time::sleep(theme::INTERACTION_TIME).await;

        self.game.close_dialog().await?;

        Ok(response)
    }

    pub fn world(&self) -> &Handle {
        self.world.as_ref().unwrap()
    }
}

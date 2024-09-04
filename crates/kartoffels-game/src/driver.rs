use crate::play::Policy;
use anyhow::{anyhow, Result};
use kartoffels_ui::{theme, Ui};
use kartoffels_world::prelude::Handle as WorldHandle;
use tokio::sync::{mpsc, oneshot};
use tokio::time;

#[derive(Debug)]
pub struct DrivenGame {
    tx: DriverEventTx,
}

impl DrivenGame {
    const ERR: &'static str = "world has crashed";

    pub fn new() -> (Self, DriverEventRx) {
        let (tx, rx) = mpsc::channel(1);
        let this = Self { tx };

        (this, rx)
    }

    pub async fn join(&self, handle: WorldHandle) -> Result<()> {
        self.send(DriverEvent::Join(handle)).await?;

        Ok(())
    }

    pub async fn pause(&self, paused: bool) -> Result<()> {
        self.send(DriverEvent::Pause(paused)).await?;

        Ok(())
    }

    pub async fn set_policy(&self, policy: Policy) -> Result<()> {
        self.send(DriverEvent::SetPolicy(policy)).await?;

        Ok(())
    }

    pub async fn update_policy(
        &self,
        f: impl FnOnce(&mut Policy) + Send + Sync + 'static,
    ) -> Result<()> {
        self.send(DriverEvent::UpdatePolicy(Box::new(f))).await?;

        Ok(())
    }

    pub async fn dialog<T>(
        &self,
        mut dialog: impl FnMut(&mut Ui, &mut Option<oneshot::Sender<T>>)
            + Send
            + Sync
            + 'static,
    ) -> Result<T>
    where
        T: Send + Sync + 'static,
    {
        let (tx, rx) = oneshot::channel();
        let mut tx = Some(tx);

        self.open_dialog(move |ui| {
            dialog(ui, &mut tx);
        })
        .await?;

        let result = rx.await?;

        time::sleep(theme::INTERACTION_TIME).await;

        self.close_dialog().await?;

        Ok(result)
    }

    pub async fn open_dialog(
        &self,
        dialog: impl FnMut(&mut Ui) + Send + Sync + 'static,
    ) -> Result<()> {
        self.send(DriverEvent::OpenDialog(Box::new(dialog))).await?;

        Ok(())
    }

    pub async fn close_dialog(&self) -> Result<()> {
        self.send(DriverEvent::CloseDialog).await?;

        Ok(())
    }

    async fn send(&self, event: DriverEvent) -> Result<()> {
        self.tx
            .send(event)
            .await
            .map_err(|_| anyhow!("{}", Self::ERR))?;

        Ok(())
    }
}

pub type DriverEventTx = mpsc::Sender<DriverEvent>;
pub type DriverEventRx = mpsc::Receiver<DriverEvent>;

pub enum DriverEvent {
    Join(WorldHandle),
    Pause(bool),
    SetPolicy(Policy),
    UpdatePolicy(Box<dyn FnOnce(&mut Policy) + Send + Sync>),
    OpenDialog(Box<dyn FnMut(&mut Ui) + Send + Sync>),
    CloseDialog,
}

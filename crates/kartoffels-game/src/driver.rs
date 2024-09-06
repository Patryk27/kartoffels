use crate::play::{HelpDialogRef, Policy};
use anyhow::{anyhow, Result};
use kartoffels_ui::Ui;
use kartoffels_world::prelude::{Handle as WorldHandle, Snapshot};
use std::task::Poll;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug)]
pub struct DrivenGame {
    tx: DriverEventTx,
}

impl DrivenGame {
    const ERR: &'static str = "lost connection to the game";

    pub fn new() -> (Self, DriverEventRx) {
        let (tx, rx) = mpsc::channel(1);
        let this = Self { tx };

        (this, rx)
    }

    pub async fn join(&self, handle: WorldHandle) -> Result<()> {
        self.send(DriverEvent::Join(handle)).await?;

        Ok(())
    }

    pub async fn pause(&self) -> Result<()> {
        self.send(DriverEvent::Pause).await?;

        Ok(())
    }

    pub async fn resume(&self) -> Result<()> {
        self.send(DriverEvent::Resume).await?;

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

    pub async fn set_help(&self, dialog: HelpDialogRef) -> Result<()> {
        self.send(DriverEvent::SetHelp(dialog)).await?;

        Ok(())
    }

    pub async fn poll<T>(
        &self,
        mut f: impl FnMut(&Snapshot) -> Poll<T> + Send + Sync + 'static,
    ) -> Result<T>
    where
        T: Send + 'static,
    {
        let (tx, rx) = oneshot::channel();
        let mut tx = Some(tx);

        self.send(DriverEvent::Poll(Box::new(move |world| {
            if let Poll::Ready(result) = f(world) {
                if let Some(tx) = tx.take() {
                    _ = tx.send(result);
                }

                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })))
        .await?;

        Ok(rx.await?)
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

#[allow(clippy::type_complexity)]
pub enum DriverEvent {
    Join(WorldHandle),
    Pause,
    Resume,
    SetPolicy(Policy),
    UpdatePolicy(Box<dyn FnOnce(&mut Policy) + Send + Sync>),
    OpenDialog(Box<dyn FnMut(&mut Ui) + Send + Sync>),
    CloseDialog,
    SetHelp(HelpDialogRef),
    Poll(Box<dyn FnMut(&Snapshot) -> Poll<()> + Send + Sync>),
}

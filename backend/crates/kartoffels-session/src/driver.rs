use crate::views::game::{HelpDialogRef, Permissions, PollCtxt, PollFn};
use anyhow::{anyhow, Result};
use kartoffels_ui::{theme, Dialog, Ui};
use kartoffels_world::prelude::Handle as WorldHandle;
use std::task::Poll;
use tokio::sync::{mpsc, oneshot};
use tokio::time;

#[derive(Debug)]
pub struct DrivenGame {
    tx: DriverEventTx,
}

impl DrivenGame {
    const ERR: &'static str = "lost connection to the game";

    pub fn new() -> (Self, DriverEventRx) {
        let (tx, rx) = mpsc::channel(4);
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

    pub async fn set_perms(&self, perms: Permissions) -> Result<()> {
        self.send(DriverEvent::SetPerms(perms)).await?;

        Ok(())
    }

    pub async fn update_perms(
        &self,
        f: impl FnOnce(&mut Permissions) + Send + 'static,
    ) -> Result<()> {
        self.send(DriverEvent::UpdatePerms(Box::new(f))).await?;

        Ok(())
    }

    pub async fn open_dialog(
        &self,
        dialog: impl FnMut(&mut Ui<()>) + Send + 'static,
    ) -> Result<()> {
        self.send(DriverEvent::OpenDialog(Box::new(dialog))).await?;

        Ok(())
    }

    pub async fn close_dialog(&self) -> Result<()> {
        self.send(DriverEvent::CloseDialog).await?;

        Ok(())
    }

    pub async fn run_dialog<T>(&self, dialog: &'static Dialog<T>) -> Result<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        let (tx, rx) = oneshot::channel();
        let mut tx = Some(tx);

        self.open_dialog(move |ui| {
            let resp = ui.catch(|ui| {
                dialog.render(ui);
            });

            if let Some(resp) = resp {
                if let Some(tx) = tx.take() {
                    _ = tx.send(resp);
                }
            }
        })
        .await?;

        let resp = rx.await?;

        time::sleep(theme::FRAME_TIME).await;

        self.close_dialog().await?;

        Ok(resp)
    }

    pub async fn set_help(&self, dialog: Option<HelpDialogRef>) -> Result<()> {
        self.send(DriverEvent::SetHelp(dialog)).await?;

        Ok(())
    }

    pub async fn set_status(&self, status: Option<String>) -> Result<()> {
        self.send(DriverEvent::SetStatus(status)).await?;

        Ok(())
    }

    pub async fn poll<T>(
        &self,
        mut f: impl FnMut(PollCtxt) -> Poll<T> + Send + 'static,
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

    pub async fn copy_to_clipboard(
        &self,
        payload: impl Into<String>,
    ) -> Result<()> {
        self.send(DriverEvent::CopyToClipboard(payload.into()))
            .await?;

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

#[allow(clippy::type_complexity)]
pub enum DriverEvent {
    Join(WorldHandle),
    Pause,
    Resume,
    SetPerms(Permissions),
    UpdatePerms(Box<dyn FnOnce(&mut Permissions) + Send>),
    OpenDialog(Box<dyn FnMut(&mut Ui<()>) + Send>),
    CloseDialog,
    SetHelp(Option<HelpDialogRef>),
    SetStatus(Option<String>),
    Poll(PollFn),
    CopyToClipboard(String),
}

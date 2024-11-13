use super::{Dialog, State};
use crate::views::game::{HelpMsgRef, Perms};
use anyhow::{anyhow, Result};
use kartoffels_ui::{theme, Msg, Term, Ui};
use kartoffels_world::prelude::Handle as WorldHandle;
use std::time::Instant;
use tokio::sync::{mpsc, oneshot};
use tokio::time;

#[derive(Debug)]
pub struct GameCtrl {
    tx: GameCtrlTx,
}

impl GameCtrl {
    const ERR: &'static str = "lost connection to the game";

    pub(super) fn new() -> (Self, GameCtrlRx) {
        let (tx, rx) = mpsc::channel(4);
        let this = Self { tx };

        (this, rx)
    }

    pub async fn join(&self, world: WorldHandle) -> Result<()> {
        self.send(GameCtrlEvent::Join(world)).await?;

        Ok(())
    }

    pub async fn pause(&self) -> Result<()> {
        self.send(GameCtrlEvent::Pause).await?;

        Ok(())
    }

    pub async fn resume(&self) -> Result<()> {
        self.send(GameCtrlEvent::Resume).await?;

        Ok(())
    }

    pub async fn set_perms(&self, perms: Perms) -> Result<()> {
        self.send(GameCtrlEvent::SetPerms(perms)).await?;

        Ok(())
    }

    pub async fn update_perms(
        &self,
        f: impl FnOnce(&mut Perms) + Send + 'static,
    ) -> Result<()> {
        self.send(GameCtrlEvent::UpdatePerms(Box::new(f))).await?;

        Ok(())
    }

    async fn open_dialog(
        &self,
        dialog: impl FnMut(&mut Ui<()>) + Send + 'static,
    ) -> Result<()> {
        self.send(GameCtrlEvent::SetDialog(Some(Box::new(dialog))))
            .await?;

        Ok(())
    }

    async fn close_dialog(&self) -> Result<()> {
        self.send(GameCtrlEvent::SetDialog(None)).await?;

        Ok(())
    }

    pub async fn show_msg<T>(&self, msg: &'static Msg<T>) -> Result<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        let (tx, rx) = oneshot::channel();
        let mut tx = Some(tx);

        self.open_dialog(move |ui| {
            let resp = ui.catch(|ui| {
                msg.render(ui);
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

    pub async fn set_help(&self, dialog: Option<HelpMsgRef>) -> Result<()> {
        self.send(GameCtrlEvent::SetHelp(dialog)).await?;

        Ok(())
    }

    pub async fn set_status(&self, status: Option<String>) -> Result<()> {
        self.send(GameCtrlEvent::SetStatus(status)).await?;

        Ok(())
    }

    pub async fn copy_to_clipboard(
        &self,
        payload: impl Into<String>,
    ) -> Result<()> {
        self.send(GameCtrlEvent::CopyToClipboard(payload.into()))
            .await?;

        Ok(())
    }

    pub async fn get_snapshot_version(&self) -> Result<u64> {
        let (tx, rx) = oneshot::channel();

        self.send(GameCtrlEvent::GetSnapshotVersion(tx)).await?;

        rx.await.map_err(|_| anyhow!("{}", Self::ERR))
    }

    async fn send(&self, event: GameCtrlEvent) -> Result<()> {
        self.tx
            .send(event)
            .await
            .map_err(|_| anyhow!("{}", Self::ERR))?;

        Ok(())
    }
}

pub(super) type GameCtrlTx = mpsc::Sender<GameCtrlEvent>;
pub(super) type GameCtrlRx = mpsc::Receiver<GameCtrlEvent>;

#[allow(clippy::type_complexity)]
pub(super) enum GameCtrlEvent {
    Join(WorldHandle),
    Pause,
    Resume,
    SetPerms(Perms),
    UpdatePerms(Box<dyn FnOnce(&mut Perms) + Send>),
    SetDialog(Option<Box<dyn FnMut(&mut Ui<()>) + Send>>),
    SetHelp(Option<HelpMsgRef>),
    SetStatus(Option<String>),
    CopyToClipboard(String),
    GetSnapshotVersion(oneshot::Sender<u64>),
}

impl GameCtrlEvent {
    pub(super) async fn handle(
        self,
        state: &mut State,
        term: &mut Term,
    ) -> Result<()> {
        match self {
            GameCtrlEvent::Join(handle) => {
                let mut snapshots = handle.snapshots();

                state.snapshot = snapshots.next().await?;
                state.snapshots = Some(snapshots);
                state.camera.set_at(state.snapshot.raw_map().center());
                state.handle = Some(handle);
                state.bot = None;
            }

            GameCtrlEvent::Pause => {
                state.pause().await?;
            }

            GameCtrlEvent::Resume => {
                state.resume().await?;
            }

            GameCtrlEvent::SetPerms(perms) => {
                state.perms = perms;
            }

            GameCtrlEvent::UpdatePerms(f) => {
                f(&mut state.perms);
            }

            GameCtrlEvent::SetDialog(dialog) => {
                state.dialog = dialog.map(Dialog::Custom);
            }

            GameCtrlEvent::SetHelp(help) => {
                state.help = help;
            }

            GameCtrlEvent::SetStatus(status) => {
                state.status = status.map(|status| (status, Instant::now()));
            }

            GameCtrlEvent::CopyToClipboard(payload) => {
                term.copy_to_clipboard(payload).await?;
            }

            GameCtrlEvent::GetSnapshotVersion(tx) => {
                _ = tx.send(state.snapshot.version());
            }
        }

        Ok(())
    }
}

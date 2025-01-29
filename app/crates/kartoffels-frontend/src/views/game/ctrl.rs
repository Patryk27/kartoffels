use super::{Modal, State};
use crate::views::game::{Config, HelpMsgRef};
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
    const ERR: &'static str = "game has crashed";

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

    pub async fn set_config(&self, config: Config) -> Result<()> {
        self.send(GameCtrlEvent::SetConfig(config)).await?;

        Ok(())
    }

    async fn open_modal(
        &self,
        modal: impl FnMut(&mut Ui<()>) + Send + 'static,
    ) -> Result<()> {
        self.send(GameCtrlEvent::SetModal(Some(Box::new(modal))))
            .await?;

        Ok(())
    }

    async fn close_modal(&self) -> Result<()> {
        self.send(GameCtrlEvent::SetModal(None)).await?;

        Ok(())
    }

    pub async fn msg<T>(&self, msg: &'static Msg<T>) -> Result<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        let (tx, rx) = oneshot::channel();
        let mut tx = Some(tx);

        self.open_modal(move |ui| {
            let event = ui.catch(|ui| {
                msg.render(ui);
            });

            if let Some(event) = event
                && let Some(tx) = tx.take()
            {
                _ = tx.send(event);
            }
        })
        .await?;

        let event = rx.await?;

        time::sleep(theme::FRAME_TIME).await;

        self.close_modal().await?;

        Ok(event)
    }

    pub async fn set_help(&self, help: Option<HelpMsgRef>) -> Result<()> {
        self.send(GameCtrlEvent::SetHelp(help)).await?;

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

    pub async fn get_world_version(&self) -> Result<u64> {
        let (tx, rx) = oneshot::channel();

        self.send(GameCtrlEvent::GetWorldVersion(tx)).await?;

        rx.await.map_err(|_| anyhow!("{}", Self::ERR))
    }

    pub async fn wait_for_restart(&self) -> Result<()> {
        let (tx, rx) = oneshot::channel();

        self.send(GameCtrlEvent::WaitForRestart(tx)).await?;

        rx.await.map_err(|_| anyhow!("{}", Self::ERR))
    }

    /// Waits for the user interface to catch up with given version of the
    /// world.
    ///
    /// This comes handy for tests so that our assertions are more reliable.
    pub async fn sync(&self, version: u64) -> Result<()> {
        loop {
            if self.get_world_version().await? >= version {
                return Ok(());
            }
        }
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
    SetConfig(Config),
    SetModal(Option<Box<dyn FnMut(&mut Ui<()>) + Send>>),
    SetHelp(Option<HelpMsgRef>),
    SetStatus(Option<String>),
    CopyToClipboard(String),
    GetWorldVersion(oneshot::Sender<u64>),
    WaitForRestart(oneshot::Sender<()>),
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
                state.camera.set(state.snapshot.tiles.center());
                state.handle = Some(handle);
                state.bot = None;
            }

            GameCtrlEvent::Pause => {
                state.pause().await?;
            }

            GameCtrlEvent::Resume => {
                state.resume().await?;
            }

            GameCtrlEvent::SetConfig(config) => {
                state.config = config;
            }

            GameCtrlEvent::SetModal(modal) => {
                state.modal = modal.map(Modal::Custom).map(Box::new);
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

            GameCtrlEvent::GetWorldVersion(tx) => {
                _ = tx.send(state.snapshot.version);
            }

            GameCtrlEvent::WaitForRestart(tx) => {
                state.config.can_join_bots = false;
                state.config.can_restart_bots = false;
                state.config.can_restart_bots = false;
                state.config.can_spawn_bots = false;
                state.config.can_upload_bots = false;
                state.restart = Some(tx);
            }
        }

        Ok(())
    }
}

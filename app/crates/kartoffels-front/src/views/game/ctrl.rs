use super::{Modal, State};
use crate::views::game::{Config, HelpMsgRef};
use crate::{theme, Frame, Msg, Ui};
use anyhow::{anyhow, Result};
use kartoffels_store::World;
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

    pub async fn join(&self, world: &World) -> Result<()> {
        self.send(GameCtrlRequest::Join(world.clone())).await?;

        Ok(())
    }

    pub async fn pause(&self) -> Result<()> {
        self.send(GameCtrlRequest::Pause).await?;

        Ok(())
    }

    pub async fn resume(&self) -> Result<()> {
        self.send(GameCtrlRequest::Resume).await?;

        Ok(())
    }

    pub async fn set_config(&self, config: Config) -> Result<()> {
        self.send(GameCtrlRequest::SetConfig(config)).await?;

        Ok(())
    }

    async fn open_modal(
        &self,
        modal: impl FnMut(&mut Ui<()>) + Send + 'static,
    ) -> Result<()> {
        self.send(GameCtrlRequest::SetModal(Some(Box::new(modal))))
            .await?;

        Ok(())
    }

    async fn close_modal(&self) -> Result<()> {
        self.send(GameCtrlRequest::SetModal(None)).await?;

        Ok(())
    }

    pub async fn msg<T>(&self, msg: &'static Msg<T>) -> Result<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        let (tx, rx) = oneshot::channel();
        let mut tx = Some(tx);

        self.open_modal(move |ui| {
            let event = ui.catching(|ui| {
                ui.add(msg);
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
        self.send(GameCtrlRequest::SetHelp(help)).await?;

        Ok(())
    }

    pub async fn set_label(&self, label: Option<String>) -> Result<()> {
        self.send(GameCtrlRequest::SetLabel(label)).await?;

        Ok(())
    }

    pub async fn copy(&self, payload: impl Into<String>) -> Result<()> {
        self.send(GameCtrlRequest::Copy(payload.into())).await?;

        Ok(())
    }

    pub async fn get_world_version(&self) -> Result<u64> {
        let (tx, rx) = oneshot::channel();

        self.send(GameCtrlRequest::GetWorldVersion(tx)).await?;

        rx.await.map_err(|_| anyhow!("{}", Self::ERR))
    }

    pub async fn wait_for_restart(&self) -> Result<()> {
        let (tx, rx) = oneshot::channel();

        self.send(GameCtrlRequest::WaitForRestart(tx)).await?;

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

    async fn send(&self, event: GameCtrlRequest) -> Result<()> {
        self.tx
            .send(event)
            .await
            .map_err(|_| anyhow!("{}", Self::ERR))?;

        Ok(())
    }
}

pub(super) type GameCtrlTx = mpsc::Sender<GameCtrlRequest>;
pub(super) type GameCtrlRx = mpsc::Receiver<GameCtrlRequest>;

#[allow(clippy::type_complexity)]
pub(super) enum GameCtrlRequest {
    Join(World),
    Pause,
    Resume,
    SetConfig(Config),
    SetModal(Option<Box<dyn FnMut(&mut Ui<()>) + Send>>),
    SetHelp(Option<HelpMsgRef>),
    SetLabel(Option<String>),
    Copy(String),
    GetWorldVersion(oneshot::Sender<u64>),
    WaitForRestart(oneshot::Sender<()>),
}

impl GameCtrlRequest {
    pub(super) async fn handle(
        self,
        state: &mut State,
        frame: &mut Frame,
    ) -> Result<()> {
        match self {
            GameCtrlRequest::Join(handle) => {
                let mut snapshots = handle.snapshots();

                state.snapshot = snapshots.next().await?;
                state.snapshots = Some(snapshots);
                state.world = Some(handle);
                state.bot = None;

                state.camera.look_at(state.snapshot.tiles.center());
            }

            GameCtrlRequest::Pause => {
                state.pause().await?;
            }

            GameCtrlRequest::Resume => {
                state.resume().await?;
            }

            GameCtrlRequest::SetConfig(config) => {
                state.config = config;
            }

            GameCtrlRequest::SetModal(modal) => {
                state.modal = modal.map(Modal::Custom).map(Box::new);
            }

            GameCtrlRequest::SetHelp(help) => {
                state.help = help;
            }

            GameCtrlRequest::SetLabel(label) => {
                state.label = label.map(|status| (status, Instant::now()));
            }

            GameCtrlRequest::Copy(payload) => {
                frame.copy(payload).await?;
            }

            GameCtrlRequest::GetWorldVersion(tx) => {
                _ = tx.send(state.snapshot.version);
            }

            GameCtrlRequest::WaitForRestart(tx) => {
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

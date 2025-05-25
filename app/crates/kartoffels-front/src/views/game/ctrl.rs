use super::{Modal, View};
use crate::views::game::{Config, HelpMsgRef};
use crate::{Frame, Msg, Ui, theme};
use anyhow::{Result, anyhow};
use kartoffels_store::World;
use std::time::Instant;
use tokio::sync::{mpsc, oneshot};
use tokio::time;
use tracing::info;

/// Controller for the game view, allows to display in-game messages etc.
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

    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
        }
    }

    pub async fn visit(&self, world: &World) -> Result<()> {
        self.send(GameCtrlRequest::Visit(world.clone())).await?;

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

    /// Shows a message.
    ///
    /// See: [`Self::msg_ex()`].
    pub async fn msg<T>(&self, msg: &'static Msg<T>) -> Result<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        self.msg_ex(msg).await?.close().await
    }

    /// Shows a message.
    ///
    /// As compared to [`Self::msg()`], this function doesn't close the message,
    /// so it comes handy for all those messages like "do you want to go back?"
    /// that mustn't auto-disappear when user answers them.
    ///
    /// (as in: if those messages disappeared, it wouldn't look nicely paired
    /// with the fade-out transition.)
    ///
    /// See: [`Self::msg()`].
    pub async fn msg_ex<T>(
        &self,
        msg: &'static Msg<T>,
    ) -> Result<GameMsgHandle<T>>
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

        let result = rx.await?;

        time::sleep(theme::FRAME_TIME).await;

        Ok(GameMsgHandle {
            ctrl: self.clone(),
            answer: result,
        })
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
    Visit(World),
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
        view: &mut View,
        frame: &mut Frame,
    ) -> Result<()> {
        match self {
            GameCtrlRequest::Visit(handle) => {
                info!(id=?handle.id(), name=?handle.name(), "visiting");

                let mut snapshots = handle.snapshots();

                view.events = handle.events().ok();
                view.snapshot = snapshots.next().await?;
                view.snapshots = Some(snapshots);
                view.world = Some(handle);
                view.bot = None;

                view.camera.look_at(view.snapshot.tiles.center());
            }

            GameCtrlRequest::Pause => {
                view.pause().await?;
            }

            GameCtrlRequest::Resume => {
                view.resume().await?;
            }

            GameCtrlRequest::SetConfig(config) => {
                view.config = config;
            }

            GameCtrlRequest::SetModal(modal) => {
                view.modal = modal.map(Modal::Custom).map(Box::new);
            }

            GameCtrlRequest::SetHelp(help) => {
                view.help = help;
            }

            GameCtrlRequest::SetLabel(label) => {
                view.label = label.map(|status| (status, Instant::now()));
            }

            GameCtrlRequest::Copy(payload) => {
                frame.copy(payload).await?;
            }

            GameCtrlRequest::GetWorldVersion(tx) => {
                _ = tx.send(view.snapshot.version);
            }

            GameCtrlRequest::WaitForRestart(tx) => {
                view.config.can_join_bots = false;
                view.config.can_kill_bots = false;
                view.config.can_spawn_bots = false;
                view.config.can_upload_bots = false;
                view.restart = Some(tx);
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct GameMsgHandle<T> {
    ctrl: GameCtrl,
    answer: T,
}

impl<T> GameMsgHandle<T> {
    pub fn answer(&self) -> &T {
        &self.answer
    }

    pub async fn close(self) -> Result<T> {
        self.ctrl.close_modal().await?;

        Ok(self.answer)
    }
}

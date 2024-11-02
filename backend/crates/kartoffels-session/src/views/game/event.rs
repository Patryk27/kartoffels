use super::{Dialog, ErrorDialog, State, UploadBotDialog};
use anyhow::Result;
use glam::IVec2;
use itertools::Either;
use kartoffels_store::{SessionId, Store};
use kartoffels_ui::Term;
use kartoffels_world::prelude::{BotId, ClockSpeed, BOT_ROBERTO};
use std::ops::ControlFlow;

#[derive(Debug)]
pub enum Event {
    CloseDialog,
    GoBack,
    JoinBot(BotId),
    MoveCamera(IVec2),
    TogglePause,
    ShowBotsDialog,
    ShowErrorDialog(String),
    ShowHelpDialog,
    ShowJoinBotDialog,
    ShowUploadBotDialog,
    ShowBotHistoryDialog,
    SpawnRoberto,
    UploadBot(Either<String, Vec<u8>>),
    LeaveBot,
    RestartBot,
    DestroyBot,
    FollowBot,
    Overclock(ClockSpeed),
    CopyToClipboard(String),
}

impl Event {
    pub async fn handle(
        self,
        store: &Store,
        sess: SessionId,
        term: &mut Term,
        state: &mut State,
    ) -> Result<ControlFlow<(), ()>> {
        match self {
            Event::CloseDialog => {
                state.dialog = None;
            }

            Event::GoBack => {
                if state.dialog.is_some() {
                    return Ok(ControlFlow::Break(()));
                } else {
                    state.dialog = Some(Dialog::Leaving(Default::default()));
                }
            }

            Event::JoinBot(id) => {
                state.join_bot(id);
                state.dialog = None;
            }

            Event::MoveCamera(delta) => {
                state.camera.animate_by(delta);

                if let Some(bot) = &mut state.bot {
                    bot.follow = false;
                }
            }

            Event::TogglePause => {
                if state.paused {
                    state.resume().await?;
                } else {
                    state.pause().await?;
                }
            }

            Event::ShowBotsDialog => {
                state.dialog = Some(Dialog::Bots(Default::default()));
            }

            Event::ShowHelpDialog => {
                state.dialog = Some(Dialog::Help(state.help.unwrap()));
            }

            Event::ShowErrorDialog(error) => {
                state.dialog = Some(Dialog::Error(ErrorDialog::new(error)));
            }

            Event::ShowJoinBotDialog => {
                state.dialog = Some(Dialog::JoinBot(Default::default()));
            }

            Event::ShowUploadBotDialog => {
                if term.ty().is_web() {
                    term.send(vec![0x04]).await?;
                }

                state.dialog =
                    Some(Dialog::UploadBot(UploadBotDialog::new(store, sess)));
            }

            Event::ShowBotHistoryDialog => {
                // TODO
            }

            Event::SpawnRoberto => {
                state
                    .upload_bot(Either::Right(BOT_ROBERTO.to_vec()))
                    .await?;
            }

            Event::UploadBot(src) => {
                state.dialog = None;
                state.upload_bot(src).await?;
            }

            Event::LeaveBot => {
                state.bot = None;
            }

            Event::RestartBot => {
                let id = state.bot.as_ref().unwrap().id;

                state
                    .handle
                    .as_ref()
                    .unwrap()
                    .kill_bot(id, "forcefully restarted")
                    .await?;
            }

            Event::DestroyBot => {
                let id = state.bot.take().unwrap().id;

                state.handle.as_ref().unwrap().destroy_bot(id).await?;
            }

            Event::FollowBot => {
                if let Some(bot) = &mut state.bot {
                    bot.follow = !bot.follow;
                }
            }

            Event::Overclock(speed) => {
                state.handle.as_ref().unwrap().overclock(speed).await?;
                state.speed = speed;
            }

            Event::CopyToClipboard(payload) => {
                term.copy_to_clipboard(payload).await?;
            }
        }

        Ok(ControlFlow::Continue(()))
    }
}

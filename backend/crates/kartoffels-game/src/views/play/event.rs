use super::{Dialog, ErrorDialog, PauseState, State};
use crate::bots;
use anyhow::Result;
use glam::IVec2;
use itertools::Either;
use kartoffels_ui::{theme, Term};
use kartoffels_world::prelude::BotId;
use std::ops::ControlFlow;
use tokio::time;

#[derive(Debug)]
pub enum Event {
    CloseDialog,
    CopyToClipboard(String),
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
    UploadBot(String),
    LeaveBot,
    RestartBot,
    DestroyBot,
    FollowBot,
}

impl Event {
    pub async fn handle(
        self,
        state: &mut State,
        term: &mut Term,
    ) -> Result<ControlFlow<(), ()>> {
        time::sleep(theme::INTERACTION_TIME).await;

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
                state.camera += delta;

                if let Some(bot) = &mut state.bot {
                    bot.is_followed = false;
                }
            }

            Event::TogglePause => match state.pause {
                PauseState::Resumed => {
                    state.pause().await?;
                }
                PauseState::Paused(_) => {
                    state.resume().await?;
                }
            },

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

                state.dialog = Some(Dialog::UploadBot(Default::default()));
            }

            Event::ShowBotHistoryDialog => {
                // TODO
            }

            Event::SpawnRoberto => {
                state
                    .upload_bot(Either::Right(bots::ROBERTO.to_vec()))
                    .await?;
            }

            Event::UploadBot(src) => {
                state.dialog = None;
                state.upload_bot(Either::Left(src)).await?;
            }

            Event::LeaveBot => {
                state.bot = None;
            }

            Event::RestartBot => {
                let id = state.bot.as_ref().unwrap().id;

                state.handle.as_ref().unwrap().restart_bot(id).await?;
            }

            Event::DestroyBot => {
                let id = state.bot.take().unwrap().id;

                state.handle.as_ref().unwrap().destroy_bot(id).await?;
            }

            Event::FollowBot => {
                if let Some(bot) = &mut state.bot {
                    bot.is_followed = !bot.is_followed;
                }
            }

            Event::CopyToClipboard(payload) => {
                term.copy_to_clipboard(payload).await?;
            }
        }

        Ok(ControlFlow::Continue(()))
    }
}

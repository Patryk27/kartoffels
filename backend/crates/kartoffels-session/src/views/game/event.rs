use super::{
    BotCount, BotLocation, BotPrefab, Dialog, ErrorDialog, Mode,
    SpawnPrefabBotDialog, State, UploadBotDialog,
};
use anyhow::Result;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use glam::IVec2;
use itertools::Either;
use kartoffels_store::{SessionId, Store};
use kartoffels_ui::Term;
use kartoffels_world::prelude::{BotId, ClockSpeed, CreateBotRequest};
use std::ops::ControlFlow;

#[derive(Debug)]
pub enum Event {
    CloseDialog,
    GoBack {
        confirm: bool,
    },
    JoinBot {
        id: BotId,
    },
    MoveCamera {
        delta: IVec2,
    },
    TogglePause,
    OpenBotsDialog,
    OpenErrorDialog {
        error: String,
    },
    OpenHelpDialog,
    OpenJoinBotDialog,
    OpenUploadBotDialog,
    OpenSpawnPrefabBotDialog,
    CreateBot {
        src: Either<String, Vec<u8>>,
        pos: Option<IVec2>,
        follow: bool,
    },
    SpawnPrefabBot {
        count: BotCount,
        prefab: BotPrefab,
        location: BotLocation,
    },
    LeaveBot,
    RestartBot,
    DestroyBot,
    FollowBot,
    Overclock(ClockSpeed),
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

            Event::GoBack { confirm } => match &state.mode {
                Mode::Default => {
                    if confirm {
                        state.dialog = Some(Dialog::GoBack(Default::default()));
                    } else {
                        return Ok(ControlFlow::Break(()));
                    }
                }

                Mode::SpawningPrefabBot { .. } => {
                    state.mode = Mode::Default;
                }
            },

            Event::JoinBot { id } => {
                state.join_bot(id, true);
                state.dialog = None;
            }

            Event::MoveCamera { delta } => {
                state.camera.move_by(delta);

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

            Event::OpenBotsDialog => {
                state.dialog = Some(Dialog::Bots(Default::default()));
            }

            Event::OpenHelpDialog => {
                state.dialog = Some(Dialog::Help(state.help.unwrap()));
            }

            Event::OpenErrorDialog { error } => {
                state.dialog = Some(Dialog::Error(ErrorDialog::new(error)));
            }

            Event::OpenJoinBotDialog => {
                state.dialog = Some(Dialog::JoinBot(Default::default()));
            }

            Event::OpenUploadBotDialog => {
                if term.ty().is_web() {
                    term.send(vec![0x04]).await?;
                }

                state.dialog =
                    Some(Dialog::UploadBot(UploadBotDialog::new(store, sess)));
            }

            Event::OpenSpawnPrefabBotDialog => {
                state.dialog = Some(Dialog::SpawnPrefabBot(
                    SpawnPrefabBotDialog::default(),
                ));
            }

            Event::CreateBot {
                src,
                pos,
                follow: join,
            } => {
                state.dialog = None;
                state.create_bot(src, pos, join).await?;
            }

            Event::SpawnPrefabBot {
                count,
                prefab,
                location,
            } => {
                state.dialog = None;
                state.spawn_prefab_bot(count, prefab, location).await?;
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
        }

        Ok(ControlFlow::Continue(()))
    }
}

impl State {
    async fn create_bot(
        &mut self,
        src: Either<String, Vec<u8>>,
        pos: Option<IVec2>,
        follow: bool,
    ) -> Result<()> {
        let src = match src {
            Either::Left(src) => {
                let src = src.trim().replace('\r', "");
                let src = src.trim().replace('\n', "");

                match BASE64_STANDARD.decode(src) {
                    Ok(src) => src,
                    Err(err) => {
                        self.dialog = Some(Dialog::Error(ErrorDialog::new(
                            format!("couldn't decode pasted content:\n\n{err}"),
                        )));

                        return Ok(());
                    }
                }
            }

            Either::Right(src) => src,
        };

        let id = self
            .handle
            .as_ref()
            .unwrap()
            .create_bot(CreateBotRequest::new(src).at(pos))
            .await;

        let id = match id {
            Ok(id) => id,

            Err(err) => {
                self.dialog =
                    Some(Dialog::Error(ErrorDialog::new(format!("{err:?}"))));

                return Ok(());
            }
        };

        self.join_bot(id, follow);

        Ok(())
    }

    async fn spawn_prefab_bot(
        &mut self,
        count: BotCount,
        prefab: BotPrefab,
        location: BotLocation,
    ) -> Result<()> {
        match location {
            BotLocation::Manual => {
                self.mode = Mode::SpawningPrefabBot {
                    prefab,
                    cursor_screen: None,
                    cursor_world: None,
                    cursor_valid: false,
                };
            }

            BotLocation::Random => {
                for _ in 0..count.get() {
                    self.create_bot(
                        Either::Right(prefab.src().to_vec()),
                        None,
                        true,
                    )
                    .await?;
                }
            }
        }

        Ok(())
    }
}

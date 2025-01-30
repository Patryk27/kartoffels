use super::{
    BotPosition, BotPrefabType, BotSource, BotSourceType, BotsModal,
    ErrorModal, GoBackModal, InspectBotModal, JoinBotModal, Modal, Mode,
    SpawnBotModal, State, UploadBotModal, UploadBotRequest,
};
use anyhow::{anyhow, Error, Result};
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use glam::IVec2;
use kartoffels_ui::Term;
use kartoffels_world::prelude::{BotId, Clock, CreateBotRequest};
use std::borrow::Cow;
use std::ops::ControlFlow;

pub enum Event {
    GoBack {
        needs_confirmation: bool,
    },
    Restart,
    JoinBot {
        id: BotId,
    },
    MoveCamera {
        delta: IVec2,
    },
    TogglePause,
    CloseModal,
    OpenModal {
        modal: Box<Modal>,
    },
    OpenBotsModal,
    OpenErrorModal {
        error: Error,
    },
    OpenHelpModal,
    OpenJoinBotModal,
    OpenUploadBotModal {
        request: UploadBotRequest<BotSourceType>,
    },
    OpenSpawnBotModal,
    UploadBot {
        request: UploadBotRequest<BotSource>,
    },
    CreateBot {
        src: BotSource,
        pos: Option<IVec2>,
        follow: bool,
    },
    LeaveBot,
    RestartBot,
    DeleteBot,
    FollowBot,
    InspectBot {
        id: BotId,
    },
    Overclock {
        clock: Clock,
    },
}

impl Event {
    pub async fn handle(
        self,
        term: &mut Term,
        state: &mut State,
    ) -> Result<ControlFlow<(), ()>> {
        match self {
            Event::GoBack { needs_confirmation } => match &state.mode {
                Mode::Default => {
                    if needs_confirmation {
                        state.modal =
                            Some(Box::new(Modal::GoBack(GoBackModal)));
                    } else {
                        return Ok(ControlFlow::Break(()));
                    }
                }

                Mode::SpawningBot { .. } => {
                    state.mode = Mode::Default;
                }
            },

            Event::Restart => {
                state.bot = None;
                state.mode = Mode::Default;
                state.modal = None;

                if state.restart.take().unwrap().send(()).is_err() {
                    return Err(anyhow!("couldn't restart the game"));
                }
            }

            Event::JoinBot { id } => {
                state.join_bot(id, true);
                state.modal = None;
            }

            Event::MoveCamera { delta } => {
                state.camera.nudge_by(delta);

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

            Event::CloseModal => {
                state.modal = None;
            }

            Event::OpenModal { modal } => {
                state.modal = Some(modal);
            }

            Event::OpenBotsModal => {
                state.modal = Some(Box::new(Modal::Bots(BotsModal::default())));
            }

            Event::OpenHelpModal => {
                state.modal = Some(Box::new(Modal::Help(state.help.unwrap())));
            }

            Event::OpenErrorModal { error } => {
                state.modal =
                    Some(Box::new(Modal::Error(ErrorModal::new(error))));
            }

            Event::OpenJoinBotModal => {
                state.modal =
                    Some(Box::new(Modal::JoinBot(JoinBotModal::default())));
            }

            Event::OpenSpawnBotModal => {
                state.modal =
                    Some(Box::new(Modal::SpawnBot(SpawnBotModal::new(
                        BotSourceType::Prefab(BotPrefabType::Roberto),
                    ))));
            }

            Event::OpenUploadBotModal { request } => match request.source {
                BotSourceType::Upload => {
                    let request = request.with_source(());

                    if term.frontend().is_web() {
                        term.send(vec![0x04]).await?;
                    }

                    state.modal = Some(Box::new(Modal::UploadBot(
                        UploadBotModal::new(request),
                    )));
                }

                BotSourceType::Prefab(source) => {
                    let request = request
                        .with_source(BotSource::BinaryRef(source.source()));

                    state.modal = None;
                    state.upload_bot(request).await?;
                }
            },

            Event::UploadBot { request } => {
                state.modal = None;
                state.upload_bot(request).await?;
            }

            Event::CreateBot { src, pos, follow } => {
                state.modal = None;
                state.create_bot(&src, pos, follow).await?;
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

            Event::DeleteBot => {
                let id = state.bot.take().unwrap().id;

                state.handle.as_ref().unwrap().delete_bot(id).await?;
            }

            Event::FollowBot => {
                if let Some(bot) = &mut state.bot {
                    bot.follow = !bot.follow;
                }
            }

            Event::InspectBot { id } => {
                state.modal = Some(Box::new(Modal::InspectBot(
                    InspectBotModal::new(id, state.modal.take()),
                )));
            }

            Event::Overclock { clock } => {
                state.handle.as_ref().unwrap().overclock(clock).await?;
            }
        }

        Ok(ControlFlow::Continue(()))
    }
}

impl State {
    async fn upload_bot(
        &mut self,
        request: UploadBotRequest<BotSource>,
    ) -> Result<()> {
        match request.position {
            BotPosition::Manual => {
                self.mode = Mode::SpawningBot {
                    source: request.source,
                    cursor_screen: None,
                    cursor_world: None,
                    cursor_valid: false,
                };
            }

            BotPosition::Random => {
                for _ in 0..request.count.get() {
                    self.create_bot(&request.source, None, true).await?;
                }
            }
        }

        Ok(())
    }

    async fn create_bot(
        &mut self,
        src: &BotSource,
        pos: Option<IVec2>,
        follow: bool,
    ) -> Result<()> {
        let src = match src {
            BotSource::Base64(src) => {
                let src = src.trim().replace('\r', "");
                let src = src.trim().replace('\n', "");

                match BASE64_STANDARD.decode(src) {
                    Ok(src) => Cow::Owned(src),

                    Err(err) => {
                        self.modal =
                            Some(Box::new(Modal::Error(ErrorModal::new(
                                anyhow!("{err}")
                                    .context("couldn't decode pasted content")
                                    .context("couldn't upload bot"),
                            ))));

                        return Ok(());
                    }
                }
            }

            BotSource::Binary(src) => Cow::Owned(src.to_owned()),
            BotSource::BinaryRef(src) => Cow::Borrowed(*src),
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
                self.modal = Some(Box::new(Modal::Error(ErrorModal::new(
                    err.context("couldn't upload bot"),
                ))));

                return Ok(());
            }
        };

        self.join_bot(id, follow);

        Ok(())
    }
}

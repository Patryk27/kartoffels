use super::{
    BotPosition, BotPrefab, BotSource, BotsModal, ErrorModal, GoBackModal,
    InspectBotModal, JoinBotModal, Modal, Mode, SpawnBotModal, UploadBotModal,
    UploadBotRequest, View,
};
use crate::Frame;
use anyhow::{anyhow, Error, Result};
use glam::IVec2;
use kartoffels_world::prelude as w;
use std::ops::ControlFlow;

pub enum Event {
    Copy {
        payload: String,
    },
    GoBack {
        confirm: bool,
    },
    Restart,
    JoinBot {
        id: w::BotId,
    },
    MoveCamera {
        delta: IVec2,
    },
    ToggleStatus,
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
        request: UploadBotRequest<BotSource>,
    },
    OpenSpawnBotModal,
    UploadBot {
        request: UploadBotRequest<Vec<u8>>,
    },
    CreateBot {
        src: Vec<u8>,
        pos: Option<IVec2>,
        follow: bool,
    },
    LeaveBot,
    KillBot,
    DeleteBot,
    FollowBot,
    InspectBot {
        id: w::BotId,
    },
    Overclock {
        clock: w::Clock,
    },
}

impl Event {
    pub async fn handle(
        self,
        frame: &mut Frame,
        view: &mut View,
    ) -> Result<ControlFlow<(), ()>> {
        match self {
            Event::Copy { payload } => {
                frame.copy(payload).await?;
            }

            Event::GoBack { confirm } => match &view.mode {
                Mode::Default => {
                    if confirm {
                        view.modal = Some(Box::new(Modal::GoBack(GoBackModal)));
                    } else {
                        return Ok(ControlFlow::Break(()));
                    }
                }

                Mode::SpawningBot { .. } => {
                    view.mode = Mode::Default;
                }
            },

            Event::Restart => {
                view.bot = None;
                view.mode = Mode::Default;
                view.modal = None;

                if view.restart.take().unwrap().send(()).is_err() {
                    return Err(anyhow!("couldn't restart the game"));
                }
            }

            Event::JoinBot { id } => {
                view.join(id, true);
                view.modal = None;
            }

            Event::MoveCamera { delta } => {
                view.camera.look_at(view.camera.pos() + delta);

                if let Some(bot) = &mut view.bot {
                    bot.follow = false;
                }
            }

            Event::ToggleStatus => {
                if view.status.is_paused() {
                    view.resume().await?;
                } else {
                    view.pause().await?;
                }
            }

            Event::CloseModal => {
                view.modal = None;
            }

            Event::OpenModal { modal } => {
                view.modal = Some(modal);
            }

            Event::OpenBotsModal => {
                view.modal = Some(Box::new(Modal::Bots(BotsModal::default())));
            }

            Event::OpenHelpModal => {
                view.modal = Some(Box::new(Modal::Help(view.help.unwrap())));
            }

            Event::OpenErrorModal { error } => {
                view.modal =
                    Some(Box::new(Modal::Error(ErrorModal::new(error))));
            }

            Event::OpenJoinBotModal => {
                view.modal =
                    Some(Box::new(Modal::JoinBot(JoinBotModal::default())));
            }

            Event::OpenSpawnBotModal => {
                view.modal = Some(Box::new(Modal::SpawnBot(
                    SpawnBotModal::new(BotSource::Prefab(BotPrefab::Roberto)),
                )));
            }

            Event::OpenUploadBotModal { request } => match request.source {
                BotSource::Upload => {
                    let request = request.with_source(());

                    if frame.ty().is_web() {
                        frame.send(vec![0x04]).await?;
                    }

                    view.modal = Some(Box::new(Modal::UploadBot(
                        UploadBotModal::new(request),
                    )));
                }

                BotSource::Prefab(source) => {
                    let request = request.with_source(source.source());

                    view.modal = None;
                    view.upload_bot(request).await?;
                }
            },

            Event::UploadBot { request } => {
                view.modal = None;
                view.upload_bot(request).await?;
            }

            Event::CreateBot { src, pos, follow } => {
                view.modal = None;
                view.create_bot(src, pos, follow).await?;
            }

            Event::LeaveBot => {
                view.bot = None;
            }

            Event::KillBot => {
                let id = view.bot.as_ref().unwrap().id;

                view.world
                    .as_ref()
                    .unwrap()
                    .kill_bot(id, "god's will")
                    .await?;

                view.resume().await?;
            }

            Event::DeleteBot => {
                let id = view.bot.take().unwrap().id;

                view.world.as_ref().unwrap().delete_bot(id).await?;
                view.resume().await?;
            }

            Event::FollowBot => {
                if let Some(bot) = &mut view.bot {
                    bot.follow = !bot.follow;
                }
            }

            Event::InspectBot { id } => {
                view.modal = Some(Box::new(Modal::InspectBot(
                    InspectBotModal::new(id, view.modal.take()),
                )));
            }

            Event::Overclock { clock } => {
                view.world.as_ref().unwrap().overclock(clock).await?;
            }
        }

        Ok(ControlFlow::Continue(()))
    }
}

impl View {
    async fn upload_bot(
        &mut self,
        request: UploadBotRequest<Vec<u8>>,
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
                    self.create_bot(request.source.clone(), None, true).await?;
                }
            }
        }

        Ok(())
    }

    async fn create_bot(
        &mut self,
        src: Vec<u8>,
        pos: Option<IVec2>,
        follow: bool,
    ) -> Result<()> {
        let id = self
            .world
            .as_ref()
            .unwrap()
            .create_bot(w::CreateBotRequest::new(src).at(pos))
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

        self.join(id, follow);

        Ok(())
    }
}

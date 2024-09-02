mod bottom;
mod dialog;
mod map;
mod side;

use self::bottom::*;
use self::dialog::*;
use self::map::*;
use self::side::*;
use crate::{theme, Clear, Term, Ui};
use anyhow::{Context, Result};
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use glam::IVec2;
use kartoffels_world::prelude::{
    BotId, Handle as WorldHandle, Snapshot as WorldSnapshot,
};
use ratatui::layout::{Constraint, Layout};
use std::ops::ControlFlow;
use std::sync::Arc;
use tokio::{select, time};
use tokio_stream::StreamExt;

pub async fn run(term: &mut Term, handle: WorldHandle) -> Result<Response> {
    let mut snapshots = handle.listen().await?;

    let snapshot = snapshots
        .next()
        .await
        .context("lost connection to the world")?;

    let mut state = State {
        camera: snapshot.map.size().as_ivec2() / 2,
        bot: None,
        dialog: None,
        paused: false,
        snapshot,
        handle,
    };

    loop {
        let resp = term.draw(|ui| state.render(ui)).await?;

        if let Some(resp) = resp {
            time::sleep(theme::INTERACTION_TIME).await;

            match state.handle(resp, term).await? {
                ControlFlow::Continue(_) => {
                    //
                }

                ControlFlow::Break(response) => {
                    return Ok(response);
                }
            }
        }

        let snapshot = select! {
            _ = term.tick() => {
                continue;
            }

            snapshot = snapshots.next() => {
                snapshot.context("lost connection to the world")?
            },
        };

        if !state.paused {
            state.snapshot = snapshot;
        }
    }
}

#[derive(Debug)]
struct State {
    camera: IVec2,
    bot: Option<JoinedBot>,
    dialog: Option<Dialog>,
    paused: bool,
    snapshot: Arc<WorldSnapshot>,
    handle: WorldHandle,
}

impl State {
    fn render(&mut self, ui: &mut Ui) -> Option<InnerResponse> {
        if let Some(bot) = &self.bot {
            if bot.follow {
                if let Some(bot) = self.snapshot.bots.alive.by_id(bot.id) {
                    self.camera = bot.pos;
                }
            }
        }

        // ---

        let [main_area, bottom_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)])
                .areas(ui.area());

        let [map_area, side_area] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(SidePanel::WIDTH),
        ])
        .areas(main_area);

        let enabled = self.dialog.is_none();

        Clear::render(ui);

        let bottom_resp = ui
            .clamp(bottom_area, |ui| {
                BottomPanel::render(ui, self.paused, enabled)
            })
            .map(InnerResponse::BottomPanel);

        let side_resp = ui
            .clamp(side_area, |ui| {
                SidePanel::render(
                    ui,
                    &self.snapshot,
                    self.bot.as_ref(),
                    enabled,
                )
            })
            .map(InnerResponse::SidePanel);

        let map_resp = ui
            .clamp(map_area, |ui| {
                MapCanvas::render(
                    ui,
                    &self.snapshot,
                    self.bot.as_ref(),
                    self.camera,
                    self.paused,
                    enabled,
                )
            })
            .map(InnerResponse::MapCanvas);

        let dialog_resp = self
            .dialog
            .as_mut()
            .and_then(|dialog| dialog.render(ui, &self.snapshot))
            .map(InnerResponse::Dialog);

        bottom_resp.or(side_resp).or(map_resp).or(dialog_resp)
    }

    async fn handle(
        &mut self,
        resp: InnerResponse,
        term: &mut Term,
    ) -> Result<ControlFlow<Response, ()>> {
        match resp {
            InnerResponse::BottomPanel(response) => match response {
                BottomPanelResponse::GoBack => {
                    return Ok(ControlFlow::Break(Response::GoBack));
                }

                BottomPanelResponse::Help => {
                    self.dialog = Some(Dialog::Help(Default::default()));
                }

                BottomPanelResponse::Pause => {
                    self.paused = !self.paused;
                }

                BottomPanelResponse::ListBots => {
                    self.dialog = Some(Dialog::Bots(Default::default()));
                }
            },

            InnerResponse::Dialog(response) => match response {
                DialogResponse::Close => {
                    self.dialog = None;
                }

                DialogResponse::JoinBot(id) => {
                    self.dialog = None;
                    self.join_bot(id);
                }

                DialogResponse::UploadBot(src) => {
                    self.dialog = None;
                    self.upload_bot(src).await?;
                }

                DialogResponse::OpenTutorial => {
                    return Ok(ControlFlow::Break(Response::OpenTutorial));
                }

                DialogResponse::Throw(err) => {
                    self.dialog = Some(Dialog::Error(ErrorDialog {
                        error: err.to_string(),
                    }));
                }
            },

            InnerResponse::MapCanvas(response) => match response {
                MapCanvasResponse::MoveCamera(delta) => {
                    self.camera += delta;

                    if let Some(bot) = &mut self.bot {
                        bot.follow = false;
                    }
                }

                MapCanvasResponse::JoinBot(id) => {
                    self.join_bot(id);
                }
            },

            InnerResponse::SidePanel(response) => match response {
                SidePanelResponse::UploadBot => {
                    if term.ty().is_http() {
                        term.send(vec![0x04]).await?;
                    }

                    self.dialog = Some(Dialog::UploadBot(Default::default()));
                }

                SidePanelResponse::JoinBot => {
                    self.dialog = Some(Dialog::JoinBot(Default::default()));
                }

                SidePanelResponse::LeaveBot => {
                    self.bot = None;
                }

                SidePanelResponse::ShowBotHistory => {
                    todo!();
                }
            },
        }

        Ok(ControlFlow::Continue(()))
    }

    fn join_bot(&mut self, id: BotId) {
        self.bot = Some(JoinedBot { id, follow: true });
        self.paused = false;
    }

    async fn upload_bot(&mut self, src: String) -> Result<()> {
        let src = src.trim().replace('\n', "");

        let src = match BASE64_STANDARD.decode(src) {
            Ok(src) => src,
            Err(err) => {
                self.dialog = Some(Dialog::Error(ErrorDialog {
                    error: format!(
                        "couldn't decode pasted content:\n\n{}",
                        err
                    ),
                }));

                return Ok(());
            }
        };

        let id = match self.handle.create_bot(src, None, false).await {
            Ok(id) => id,

            Err(err) => {
                self.dialog = Some(Dialog::Error(ErrorDialog {
                    error: err.to_string(),
                }));

                return Ok(());
            }
        };

        self.join_bot(id);

        Ok(())
    }
}

#[derive(Debug)]
struct JoinedBot {
    id: BotId,
    follow: bool,
}

#[derive(Debug)]
enum InnerResponse {
    BottomPanel(BottomPanelResponse),
    Dialog(DialogResponse),
    MapCanvas(MapCanvasResponse),
    SidePanel(SidePanelResponse),
}

#[derive(Debug)]
pub enum Response {
    OpenTutorial,
    GoBack,
}

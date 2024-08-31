mod bottom_panel;
mod dialog;
mod map_canvas;
mod side_panel;

use self::bottom_panel::*;
use self::dialog::*;
use self::map_canvas::*;
use self::side_panel::*;
use crate::{theme, Clear, Term, Ui};
use anyhow::{Context, Result};
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use glam::IVec2;
use kartoffels_world::prelude::{BotId, Handle as WorldHandle, Snapshot};
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
        let response = term.draw(|ui| state.render(ui)).await?;

        if let Some(response) = response {
            time::sleep(theme::INTERACTION_TIME).await;

            match state.handle(response).await? {
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
    snapshot: Arc<Snapshot>,
    handle: WorldHandle,
}

impl State {
    fn render(&mut self, ui: &mut Ui) -> Option<InnerResponse> {
        if let Some(bot) = &self.bot {
            if bot.follow_with_camera {
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

        let bottom_panel_event = ui
            .clamp(bottom_area, |ui| {
                BottomPanel::render(ui, self.paused, enabled)
            })
            .map(InnerResponse::BottomPanel);

        let side_panel_event = ui
            .clamp(side_area, |ui| {
                SidePanel::render(
                    ui,
                    &self.snapshot,
                    self.bot.as_ref(),
                    enabled,
                )
            })
            .map(InnerResponse::SidePanel);

        let map_canvas_event = ui
            .clamp(map_area, |ui| {
                MapCanvas::render(
                    ui,
                    &self.snapshot,
                    self.camera,
                    self.paused,
                    enabled,
                )
            })
            .map(InnerResponse::MapCanvas);

        let dialog_event = self
            .dialog
            .as_mut()
            .and_then(|dialog| dialog.render(ui, &self.snapshot))
            .map(InnerResponse::Dialog);

        bottom_panel_event
            .or(side_panel_event)
            .or(map_canvas_event)
            .or(dialog_event)
    }

    async fn handle(
        &mut self,
        event: InnerResponse,
    ) -> Result<ControlFlow<Response, ()>> {
        match event {
            InnerResponse::BottomPanel(event) => match event {
                BottomPanelResponse::Quit => {
                    return Ok(ControlFlow::Break(Response::Quit));
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

            InnerResponse::Dialog(event) => match event {
                DialogResponse::Close => {
                    self.dialog = None;
                }

                DialogResponse::JoinBot(id) => {
                    self.join_bot(id);
                }

                DialogResponse::UploadBot(src) => {
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

            InnerResponse::MapCanvas(event) => match event {
                MapCanvasResponse::MoveCamera(delta) => {
                    self.camera += delta;
                }
            },

            InnerResponse::SidePanel(event) => match event {
                SidePanelResponse::UploadBot => {
                    self.dialog = Some(Dialog::UploadBot(Default::default()));
                }

                SidePanelResponse::JoinBot => {
                    self.dialog = Some(Dialog::JoinBot(Default::default()));
                }

                SidePanelResponse::LeaveBot => {
                    self.bot = None;
                }
            },
        }

        Ok(ControlFlow::Continue(()))
    }

    fn join_bot(&mut self, id: BotId) {
        self.bot = Some(JoinedBot {
            id,
            follow_with_camera: true,
        });
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
    follow_with_camera: bool,
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
    Quit,
}

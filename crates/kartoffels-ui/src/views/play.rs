mod bottom_panel;
mod dialog;
mod map_canvas;
mod side_panel;

use self::bottom_panel::*;
use self::dialog::*;
use self::map_canvas::*;
use self::side_panel::*;
use crate::{Clear, Term, Ui};
use anyhow::{Context, Result};
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use glam::IVec2;
use kartoffels_world::prelude::{BotId, Handle as WorldHandle, Snapshot};
use ratatui::layout::{Constraint, Layout};
use std::ops::ControlFlow;
use std::sync::Arc;
use tokio::select;
use tokio_stream::StreamExt;

pub async fn run(term: &mut Term, handle: WorldHandle) -> Result<Outcome> {
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
        let event = term.draw(|ui| state.render(ui)).await?;

        if let Some(event) = event {
            if let ControlFlow::Break(outcome) = state.handle(event).await? {
                return Ok(outcome);
            }
        }

        state.snapshot = select! {
            _ = term.tick() => {
                continue;
            }

            snapshot = snapshots.next() => {
                snapshot.context("lost connection to the world")?
            },
        };
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
    fn render(&mut self, ui: &mut Ui) -> Option<Event> {
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
            .map(Event::BottomPanel);

        let side_panel_event = ui
            .clamp(side_area, |ui| {
                SidePanel::render(
                    ui,
                    &self.snapshot,
                    self.bot.as_ref(),
                    enabled,
                )
            })
            .map(Event::SidePanel);

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
            .map(Event::MapCanvas);

        let dialog_event = self
            .dialog
            .as_mut()
            .and_then(|dialog| dialog.render(ui, &self.snapshot))
            .map(Event::Dialog);

        bottom_panel_event
            .or(side_panel_event)
            .or(map_canvas_event)
            .or(dialog_event)
    }

    async fn handle(
        &mut self,
        event: Event,
    ) -> Result<ControlFlow<Outcome, ()>> {
        match event {
            Event::BottomPanel(event) => match event {
                BottomPanelEvent::Quit => {
                    return Ok(ControlFlow::Break(Outcome::Quit));
                }

                BottomPanelEvent::Help => {
                    self.dialog = Some(Dialog::Help(Default::default()));
                }

                BottomPanelEvent::Pause => {
                    self.paused = !self.paused;
                }

                BottomPanelEvent::ListBots => {
                    self.dialog = Some(Dialog::Bots(Default::default()));
                }
            },

            Event::Dialog(event) => match event {
                DialogEvent::Close => {
                    self.dialog = None;
                }

                DialogEvent::JoinBot(id) => {
                    self.join_bot(id);
                }

                DialogEvent::UploadBot(src) => {
                    self.upload_bot(src).await?;
                }

                DialogEvent::OpenTutorial => {
                    return Ok(ControlFlow::Break(Outcome::OpenTutorial));
                }

                DialogEvent::Throw(err) => {
                    self.dialog = Some(Dialog::Error(ErrorDialog {
                        error: err.to_string(),
                    }));
                }
            },

            Event::MapCanvas(event) => match event {
                MapCanvasEvent::MoveCamera(delta) => {
                    self.camera += delta;
                }
            },

            Event::SidePanel(event) => match event {
                SidePanelEvent::UploadBot => {
                    self.dialog = Some(Dialog::UploadBot(Default::default()));
                }

                SidePanelEvent::JoinBot => {
                    self.dialog = Some(Dialog::JoinBot(Default::default()));
                }

                SidePanelEvent::LeaveBot => {
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
enum Event {
    BottomPanel(BottomPanelEvent),
    Dialog(DialogEvent),
    MapCanvas(MapCanvasEvent),
    SidePanel(SidePanelEvent),
}

#[derive(Debug)]
pub enum Outcome {
    OpenTutorial,
    Quit,
}

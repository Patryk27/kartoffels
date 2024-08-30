mod bottom_panel;
mod dialog;
mod map_canvas;
mod side_panel;

use self::bottom_panel::*;
use self::dialog::*;
use self::map_canvas::*;
use self::side_panel::*;
use crate::{Clear, Term};
use anyhow::{Context, Result};
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use glam::IVec2;
use itertools::Either;
use kartoffels_world::prelude::{
    BotId, Handle as WorldHandle, Snapshot as WorldSnapshot,
};
use ratatui::layout::{Constraint, Layout};
use ratatui::prelude::{Buffer, Rect};
use ratatui::widgets::Widget;
use std::future;
use std::ops::ControlFlow;
use std::sync::Arc;
use termwiz::input::InputEvent;
use tokio::select;
use tokio_stream::StreamExt;

pub async fn run(term: &mut Term, handle: WorldHandle) -> Result<Outcome> {
    let mut snapshots = handle.listen().await?;

    let snapshot = snapshots
        .next()
        .await
        .context("lost connection to the world")?;

    let mut view = View {
        camera: snapshot.map.size().as_ivec2() / 2,
        bot: Default::default(),
        dialog: Default::default(),
        paused: Default::default(),
        snapshot,
        handle,
    };

    loop {
        term.draw(|f| {
            view.render(f.area(), f.buffer_mut());
        })
        .await?;

        let msg = select! {
            event = term.read() => Either::Left(event?),
            snapshot = snapshots.next() => Either::Right(snapshot),
            _ = view.tick() => Either::Left(None),
        };

        match msg {
            Either::Left(Some(event)) => {
                match view.handle_input(event, term).await? {
                    ControlFlow::Continue(_) => {
                        continue;
                    }
                    ControlFlow::Break(outcome) => {
                        return Ok(outcome);
                    }
                }
            }

            Either::Left(None) => {
                //
            }

            Either::Right(snapshot) => {
                view.handle_snapshot(snapshot)?;
            }
        }
    }
}

#[derive(Debug)]
struct View {
    camera: IVec2,
    bot: Option<JoinedBot>,
    dialog: Option<Dialog>,
    paused: bool,
    snapshot: Arc<WorldSnapshot>,
    handle: WorldHandle,
}

impl View {
    fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let [main_area, bottom_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)])
                .areas(area);

        let [map_area, side_area] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(SidePanel::WIDTH),
        ])
        .areas(main_area);

        if let Some(bot) = &self.bot {
            if bot.follow_with_camera {
                if let Some(bot) = self.snapshot.bots.alive.by_id(bot.id) {
                    self.camera = bot.pos;
                }
            }
        }

        Clear.render(area, buf);

        BottomPanel {
            paused: self.paused,
            enabled: self.is_enabled(),
        }
        .render(bottom_area, buf);

        MapCanvas {
            snapshot: &self.snapshot,
            camera: self.camera,
            paused: self.paused,
            enabled: self.is_enabled(),
        }
        .render(map_area, buf);

        SidePanel {
            snapshot: &self.snapshot,
            bot: self.bot.as_ref(),
            enabled: self.is_enabled(),
        }
        .render(side_area, buf);

        if let Some(dialog) = &mut self.dialog {
            dialog.render(area, buf, &self.snapshot);
        }
    }

    async fn handle_input(
        &mut self,
        mut event: InputEvent,
        term: &Term,
    ) -> Result<ControlFlow<Outcome, ()>> {
        if let Some(dialog) = &mut self.dialog {
            match dialog.handle(event, &self.snapshot) {
                Some(DialogEvent::Close) => {
                    self.dialog = None;
                }

                Some(DialogEvent::JoinBot(id)) => {
                    self.dialog = None;
                    self.handle_join_bot(id);
                }

                Some(DialogEvent::UploadBot(src)) => {
                    self.dialog = None;
                    self.handle_upload_bot(src).await?;
                }

                Some(DialogEvent::OpenTutorial) => {
                    return Ok(ControlFlow::Break(Outcome::OpenTutorial));
                }

                Some(DialogEvent::Throw(error)) => {
                    self.dialog = Some(Dialog::Error(ErrorDialog { error }));
                }

                None => {
                    //
                }
            }

            return Ok(ControlFlow::Continue(()));
        }

        event = match SidePanel::handle(self.bot.is_some(), event) {
            SidePanelEvent::UploadBot => {
                self.dialog = Some(Dialog::UploadBot(Default::default()));

                return Ok(ControlFlow::Continue(()));
            }

            SidePanelEvent::JoinBot => {
                self.dialog = Some(Dialog::JoinBot(Default::default()));

                return Ok(ControlFlow::Continue(()));
            }

            SidePanelEvent::LeaveBot => {
                self.bot = None;

                return Ok(ControlFlow::Continue(()));
            }

            SidePanelEvent::Forward(event) => event,
        };

        event = match MapCanvas::handle(event, term) {
            MapCanvasEvent::MoveCamera(delta) => {
                self.camera += delta;

                return Ok(ControlFlow::Continue(()));
            }

            MapCanvasEvent::Forward(event) => event,
        };

        match BottomPanel::handle(event, self.paused) {
            Some(BottomPanelOutcome::Quit) => {
                return Ok(ControlFlow::Break(Outcome::Quit));
            }

            Some(BottomPanelOutcome::Help) => {
                self.dialog = Some(Dialog::Help(Default::default()));
            }

            Some(BottomPanelOutcome::Pause) => {
                self.paused = !self.paused;
            }

            Some(BottomPanelOutcome::ListBots) => {
                self.dialog = Some(Dialog::Bots(Default::default()));
            }

            None => (),
        }

        Ok(ControlFlow::Continue(()))
    }

    fn handle_snapshot(
        &mut self,
        snapshot: Option<Arc<WorldSnapshot>>,
    ) -> Result<()> {
        let snapshot = snapshot.context("lost connection to the world")?;

        if !self.paused {
            self.snapshot = snapshot;
        }

        Ok(())
    }

    async fn handle_upload_bot(&mut self, src: String) -> Result<()> {
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

        self.handle_join_bot(id);

        Ok(())
    }

    fn handle_join_bot(&mut self, id: BotId) {
        self.bot = Some(JoinedBot {
            id,
            follow_with_camera: true,
        });
    }

    async fn tick(&mut self) {
        if let Some(dialog) = &mut self.dialog {
            dialog.tick().await;
        } else {
            future::pending::<()>().await;
        }
    }

    fn is_enabled(&self) -> bool {
        self.dialog.is_none()
    }
}

#[derive(Debug)]
struct JoinedBot {
    id: BotId,
    follow_with_camera: bool,
}

#[derive(Debug)]
pub enum Outcome {
    OpenTutorial,
    Quit,
}

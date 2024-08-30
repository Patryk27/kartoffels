mod bottom;
mod dialog;
mod map;
mod side;

use self::bottom::*;
use self::dialog::*;
use self::map::*;
use self::side::*;
use crate::{Clear, Term};
use anyhow::{Context, Result};
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use glam::IVec2;
use itertools::Either;
use kartoffels_world::prelude::{BotId, Handle as WorldHandle, Update};
use ratatui::layout::{Constraint, Layout};
use ratatui::prelude::{Buffer, Rect};
use ratatui::widgets::Widget;
use std::ops::ControlFlow;
use std::sync::Arc;
use termwiz::input::InputEvent;
use tokio::select;
use tokio_stream::StreamExt;

pub async fn run(term: &mut Term, world: WorldHandle) -> Result<Outcome> {
    let mut updates = world.listen().await?;

    let update = updates
        .next()
        .await
        .context("lost connection to the world")?;

    let mut view = View {
        camera: update.map.size().as_ivec2() / 2,
        bot: Default::default(),
        dialog: Default::default(),
        paused: Default::default(),
        update,
        world,
    };

    loop {
        term.draw(|f| {
            view.render(f.area(), f.buffer_mut());
        })
        .await?;

        let msg = select! {
            event = term.read() => Either::Left(event?),
            update = updates.next() => Either::Right(update),
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

            Either::Right(update) => {
                view.handle_update(update)?;
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
    update: Arc<Update>,
    world: WorldHandle,
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
            if bot.is_followed {
                if let Some(bot) = self.update.bots.by_id(bot.id) {
                    self.camera = bot.pos.unwrap_or(self.camera);
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
            update: &self.update,
            camera: self.camera,
            paused: self.paused,
            enabled: self.is_enabled(),
        }
        .render(map_area, buf);

        SidePanel {
            update: &self.update,
            bot: self.bot.as_ref(),
            enabled: self.is_enabled(),
        }
        .render(side_area, buf);

        if let Some(dialog) = &mut self.dialog {
            dialog.render(area, buf, &self.update);
        }
    }

    async fn handle_input(
        &mut self,
        mut event: InputEvent,
        term: &Term,
    ) -> Result<ControlFlow<Outcome, ()>> {
        if let Some(dialog) = &mut self.dialog {
            match dialog.handle(event, &self.update) {
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
                Ok(ControlFlow::Break(Outcome::Quit))
            }

            Some(BottomPanelOutcome::Pause) => {
                self.paused = !self.paused;

                Ok(ControlFlow::Continue(()))
            }

            Some(BottomPanelOutcome::Help) => {
                self.dialog = Some(Dialog::Help(Default::default()));

                Ok(ControlFlow::Continue(()))
            }

            None => Ok(ControlFlow::Continue(())),
        }
    }

    fn handle_update(&mut self, update: Option<Arc<Update>>) -> Result<()> {
        let update = update.context("lost connection to the world")?;

        if !self.paused {
            self.update = update;
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

        let id = match self.world.create_bot(src, None, false).await {
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
            is_followed: true,
        });
    }

    async fn tick(&mut self) {
        if let Some(dialog) = &mut self.dialog {
            dialog.tick().await;
        }
    }

    fn is_enabled(&self) -> bool {
        self.dialog.is_none()
    }
}

#[derive(Debug)]
struct JoinedBot {
    id: BotId,
    is_followed: bool,
}

#[derive(Debug)]
pub enum Outcome {
    OpenTutorial,
    Quit,
}

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
use glam::ivec2;
use itertools::Either;
use kartoffels_world::prelude::{Handle as WorldHandle, Update};
use ratatui::layout::{Constraint, Layout};
use ratatui::prelude::{Buffer, Rect};
use ratatui::widgets::Widget;
use std::ops::ControlFlow;
use std::sync::Arc;
use termwiz::input::InputEvent;
use tokio::select;
use tokio_stream::StreamExt;

pub async fn run(term: &mut Term, world: WorldHandle) -> Result<()> {
    let mut updates = world.listen().await?;

    let state = updates
        .next()
        .await
        .context("lost connection to the world")?;

    let map = MapCanvas {
        camera: state.map.size().as_ivec2() / 2,
        camera_offset: Default::default(),
    };

    let mut view = View {
        state,
        map,
        side: Default::default(),
        dialog: Default::default(),
        paused: false,
    };

    loop {
        let screen_size = term
            .draw(|f| {
                f.render_widget(&mut view, f.area());
            })
            .await?;

        view.map.camera_offset =
            ivec2(screen_size.width as i32, screen_size.height as i32) / 8;

        let msg = select! {
            event = term.read() => Either::Left(event?),
            update = updates.next() => Either::Right(update),
        };

        match msg {
            Either::Left(Some(event)) => {
                match view.handle_input(&world, event).await? {
                    ControlFlow::Continue(_) => {
                        continue;
                    }
                    ControlFlow::Break(_) => {
                        return Ok(());
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
    state: Arc<Update>,
    map: MapCanvas,
    side: SidePanel,
    dialog: Option<Dialog>,
    paused: bool,
}

impl View {
    async fn handle_input(
        &mut self,
        world: &WorldHandle,
        mut event: InputEvent,
    ) -> Result<ControlFlow<(), ()>> {
        if let Some(dialog) = &mut self.dialog {
            match dialog.handle(event) {
                DialogOutcome::Close => {
                    self.dialog = None;
                }

                DialogOutcome::Upload(src) => {
                    self.dialog = None;

                    let src = src.trim().replace('\n', "");

                    let src = match BASE64_STANDARD.decode(src) {
                        Ok(src) => src,
                        Err(err) => {
                            self.dialog = Some(Dialog::Error(ErrorDialog {
                                text: err.to_string(),
                            }));

                            return Ok(ControlFlow::Continue(()));
                        }
                    };

                    let id = match world.create_bot(src, None, false).await {
                        Ok(id) => id,

                        Err(err) => {
                            self.dialog = Some(Dialog::Error(ErrorDialog {
                                text: err.to_string(),
                            }));

                            return Ok(ControlFlow::Continue(()));
                        }
                    };

                    self.side = SidePanel::Connected(ConnectedSidePanel { id });
                }

                DialogOutcome::None => {
                    //
                }
            }

            return Ok(ControlFlow::Continue(()));
        }

        event = match self.side.handle(event) {
            SidePanelOutcome::UploadBot => {
                self.dialog = Some(Dialog::Upload(UploadDialog));

                return Ok(ControlFlow::Continue(()));
            }

            SidePanelOutcome::None => {
                return Ok(ControlFlow::Continue(()));
            }

            SidePanelOutcome::Forward(event) => event,
        };

        event = match self.map.handle(event) {
            MapCanvasOutcome::None => {
                return Ok(ControlFlow::Continue(()));
            }

            MapCanvasOutcome::Forward(event) => event,
        };

        match BottomPanel::handle(event, self.paused) {
            BottomPanelOutcome::Quit => Ok(ControlFlow::Break(())),

            BottomPanelOutcome::Pause => {
                self.paused = !self.paused;

                Ok(ControlFlow::Continue(()))
            }

            BottomPanelOutcome::Help => {
                self.dialog = Some(Dialog::Help);

                Ok(ControlFlow::Continue(()))
            }

            BottomPanelOutcome::Forward => Ok(ControlFlow::Continue(())),
        }
    }

    fn handle_update(&mut self, state: Option<Arc<Update>>) -> Result<()> {
        let state = state.context("lost connection to the world")?;

        if !self.paused {
            self.state = state;
        }

        Ok(())
    }

    fn is_interface_enabled(&self) -> bool {
        self.dialog.as_ref().map_or(true, |dialog| {
            if let Dialog::Upload(_) | Dialog::Error(_) = dialog {
                false
            } else {
                true
            }
        })
    }
}

impl Widget for &mut View {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [main_area, bottom_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)])
                .areas(area);

        let [map_area, side_area] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(SidePanel::WIDTH),
        ])
        .areas(main_area);

        Clear.render(area, buf);

        BottomPanel {
            paused: self.paused,
            enabled: self.is_interface_enabled(),
        }
        .render(bottom_area, buf);

        self.map.render(
            map_area,
            buf,
            &self.state.map,
            &self.state.bots,
            self.paused,
            self.is_interface_enabled(),
        );

        self.side.render(
            side_area,
            buf,
            &self.state.bots,
            self.is_interface_enabled(),
        );

        if let Some(dialog) = &mut self.dialog {
            dialog.render(area, buf, &self.state.bots);
        }
    }
}

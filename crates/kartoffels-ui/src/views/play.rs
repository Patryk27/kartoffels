mod bottom;
mod ctrl;
mod dialog;
mod map;
mod side;

use self::bottom::*;
pub use self::ctrl::*;
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

pub async fn run(
    term: &mut Term,
    handle: WorldHandle,
    ctrl: Controller,
) -> Result<Response> {
    let mut snapshots = handle.listen().await?;

    let snapshot = snapshots
        .next()
        .await
        .context("lost connection to the world")?;

    let mut state = State {
        ctrl,
        camera: snapshot.map().size().as_ivec2() / 2,
        bot: None,
        dialog: None,
        paused: false,
        snapshot,
        handle,
    };

    loop {
        let mut resp = None;

        term.draw(|ui| {
            resp = state.render(ui);
        })
        .await?;

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
            result = term.tick() => {
                result?;
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
    ctrl: Controller,
    camera: IVec2,
    bot: Option<JoinedBot>,
    dialog: Option<Dialog>,
    paused: bool,
    snapshot: Arc<WorldSnapshot>,
    handle: WorldHandle,
}

impl State {
    fn render(&mut self, ui: &mut Ui) -> Option<StateResponse> {
        if let Some(bot) = &self.bot {
            if bot.is_followed {
                if let Some(bot) = self.snapshot.bots().alive().by_id(bot.id) {
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
                BottomPanel::render(ui, &self.ctrl, self.paused, enabled)
            })
            .map(StateResponse::BottomPanel);

        let side_resp = ui
            .clamp(side_area, |ui| {
                SidePanel::render(
                    ui,
                    &self.ctrl,
                    &self.snapshot,
                    self.bot.as_ref(),
                    enabled,
                )
            })
            .map(StateResponse::SidePanel);

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
            .map(StateResponse::MapCanvas);

        let dialog_resp = self
            .dialog
            .as_mut()
            .and_then(|dialog| dialog.render(ui, &self.snapshot))
            .map(StateResponse::Dialog);

        bottom_resp.or(side_resp).or(map_resp).or(dialog_resp)
    }

    async fn handle(
        &mut self,
        resp: StateResponse,
        term: &mut Term,
    ) -> Result<ControlFlow<Response, ()>> {
        match resp {
            StateResponse::BottomPanel(resp) => resp.handle(self).await,
            StateResponse::Dialog(resp) => resp.handle(self).await,
            StateResponse::MapCanvas(resp) => resp.handle(self),
            StateResponse::SidePanel(resp) => resp.handle(self, term).await,
        }
    }

    fn join_bot(&mut self, id: BotId) {
        self.bot = Some(JoinedBot {
            id,
            is_followed: true,
        });

        self.paused = false;
    }

    async fn upload_bot(&mut self, src: String) -> Result<()> {
        let src = src.trim().replace('\n', "");

        let src = match BASE64_STANDARD.decode(src) {
            Ok(src) => src,
            Err(err) => {
                self.dialog = Some(Dialog::Error(ErrorDialog::new(format!(
                    "couldn't decode pasted content:\n\n{}",
                    err
                ))));

                return Ok(());
            }
        };

        let id = match self.handle.create_bot(src, None, false).await {
            Ok(id) => id,

            Err(err) => {
                self.dialog =
                    Some(Dialog::Error(ErrorDialog::new(format!("{:?}", err))));

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
    is_followed: bool,
}

#[derive(Debug)]
enum StateResponse {
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

mod bottom;
mod dialog;
mod map;
mod policy;
mod side;

use self::bottom::*;
use self::dialog::*;
use self::map::*;
pub use self::policy::*;
use self::side::*;
use crate::{DriverEvent, DriverEventRx};
use anyhow::{Context, Result};
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use futures_util::{stream, Stream};
use glam::IVec2;
use itertools::Either;
use kartoffels_ui::{theme, Clear, Term, Ui};
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
    mut driver: DriverEventRx,
) -> Result<Response> {
    let mut state = State::default();

    let mut snapshots: Box<dyn Stream<Item = _> + Send + Unpin> =
        Box::new(stream::pending());

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
                    continue;
                }
                ControlFlow::Break(resp) => {
                    return Ok(resp);
                }
            }
        }

        let event = select! {
            result = term.tick() => {
                #[allow(clippy::question_mark)]
                if let Err(err) = result {
                    return Err(err);
                }

                continue;
            }

            snapshot = snapshots.next() => {
                Either::Left(snapshot.context("world has crashed")?)
            },

            Some(event) = driver.recv() => {
                Either::Right(event)
            }
        };

        match event {
            Either::Left(snapshot) => {
                if !state.paused || state.snapshot.is_default() {
                    state.snapshot = snapshot;
                }
            }

            Either::Right(event) => match event {
                DriverEvent::Join(handle) => {
                    snapshots = Box::new(handle.listen().await?);

                    let snapshot = snapshots
                        .next()
                        .await
                        .context("lost connection to the world")?;

                    state.camera = snapshot.map().size().as_ivec2() / 2;
                    state.snapshot = snapshot;
                    state.handle = Some(handle);
                }

                DriverEvent::Pause(paused) => {
                    state.pause(paused).await?;
                }

                DriverEvent::SetPolicy(policy) => {
                    state.policy = policy;
                }

                DriverEvent::UpdatePolicy(f) => {
                    f(&mut state.policy);
                }

                DriverEvent::OpenDialog(dialog) => {
                    state.dialog = Some(Dialog::Custom(dialog));
                }

                DriverEvent::CloseDialog => {
                    state.dialog = None;
                }
            },
        }
    }
}

#[derive(Default)]
struct State {
    camera: IVec2,
    bot: Option<JoinedBot>,
    dialog: Option<Dialog>,
    paused: bool,
    policy: Policy,
    snapshot: Arc<WorldSnapshot>,
    handle: Option<WorldHandle>,
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
                BottomPanel::render(
                    ui,
                    &self.policy,
                    self.handle.as_ref(),
                    self.paused,
                    enabled,
                )
            })
            .map(StateResponse::BottomPanel);

        let side_resp = ui
            .clamp(side_area, |ui| {
                SidePanel::render(
                    ui,
                    &self.policy,
                    &self.snapshot,
                    self.bot.as_ref(),
                    enabled && self.handle.is_some(),
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
                    enabled && self.handle.is_some(),
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

    async fn pause(&mut self, paused: bool) -> Result<()> {
        self.paused = paused;

        if self.policy.pause_is_propagated {
            if let Some(handle) = &self.handle {
                handle.pause(self.paused).await?;
            }
        }

        Ok(())
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

        let id = match self
            .handle
            .as_ref()
            .unwrap()
            .create_bot(src, None, false)
            .await
        {
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

mod bottom;
mod dialog;
mod map;
mod perms;
mod side;

use self::bottom::*;
use self::dialog::*;
pub use self::dialog::{HelpDialog, HelpDialogRef, HelpDialogResponse};
use self::map::*;
pub use self::perms::*;
use self::side::*;
use crate::{DriverEvent, DriverEventRx};
use anyhow::Result;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use futures_util::{stream, Stream};
use glam::IVec2;
use itertools::Either;
use kartoffels_ui::{theme, Clear, Term, Ui};
use kartoffels_world::prelude::{
    BotId, Handle as WorldHandle, Snapshot as WorldSnapshot, SnapshotStream,
    SnapshotStreamExt,
};
use ratatui::layout::{Constraint, Layout};
use std::ops::ControlFlow;
use std::sync::Arc;
use std::task::Poll;
use tokio::{select, time};

pub async fn run(term: &mut Term, mut driver: DriverEventRx) -> Result<()> {
    let mut state = State::default();

    let mut snapshots: Box<
        dyn Stream<Item = Arc<WorldSnapshot>> + Send + Unpin,
    > = Box::new(stream::pending());

    loop {
        let mut resp = None;

        term.draw(|ui| {
            resp = state.render(ui);
        })
        .await?;

        if let Some(resp) = resp {
            match state.handle_response(resp, term).await? {
                ControlFlow::Continue(_) => {
                    continue;
                }
                ControlFlow::Break(_) => {
                    return Ok(());
                }
            }
        }

        let event = select! {
            snapshot = snapshots.next_or_err() => {
                Some(Either::Left(snapshot?))
            },

            Some(event) = driver.recv() => {
                Some(Either::Right(event))
            }

            result = term.poll() => {
                #[allow(clippy::question_mark)]
                if let Err(err) = result {
                    return Err(err);
                }

                None
            }
        };

        match event {
            Some(Either::Left(snapshot)) => {
                state.handle_snapshot(snapshot);
            }

            Some(Either::Right(event)) => {
                if let Some(new_snapshots) = state.handle_event(event).await? {
                    snapshots = Box::new(new_snapshots);
                }
            }

            None => {
                //
            }
        }

        state.poll();
    }
}

#[derive(Default)]
struct State {
    bot: Option<JoinedBot>,
    camera: IVec2,
    dialog: Option<Dialog>,
    handle: Option<WorldHandle>,
    help: Option<HelpDialogRef>,
    paused: bool,
    perms: Permissions,
    poll: Option<PollFn>,
    snapshot: Arc<WorldSnapshot>,
    status: Option<String>,
}

impl State {
    fn render(&mut self, ui: &mut Ui) -> Option<Response> {
        let [main_area, bottom_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)])
                .areas(ui.area());

        let [map_area, side_area] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(SidePanel::WIDTH),
        ])
        .areas(main_area);

        Clear::render(ui);

        let mut resp = None;

        if let Some(bot) = &self.bot {
            if bot.is_followed {
                if let Some(bot) = self.snapshot.bots().alive().by_id(bot.id) {
                    self.camera = bot.pos;
                }
            }
        }

        ui.enable(self.dialog.is_none(), |ui| {
            if let Some(inner_resp) =
                ui.clamp(bottom_area, |ui| BottomPanel::render(ui, self))
            {
                resp = Some(Response::BottomPanel(inner_resp));
            }

            if let Some(inner_resp) =
                ui.clamp(side_area, |ui| SidePanel::render(ui, self))
            {
                resp = Some(Response::SidePanel(inner_resp));
            }

            if let Some(inner_resp) =
                ui.clamp(map_area, |ui| Map::render(ui, self))
            {
                resp = Some(Response::Map(inner_resp));
            }
        });

        if let Some(dialog) = &mut self.dialog {
            if let Some(inner_resp) = dialog.render(ui, &self.snapshot) {
                resp = Some(Response::Dialog(inner_resp));
            }
        }

        resp
    }

    async fn handle_response(
        &mut self,
        resp: Response,
        term: &mut Term,
    ) -> Result<ControlFlow<(), ()>> {
        time::sleep(theme::INTERACTION_TIME).await;

        match resp {
            Response::BottomPanel(resp) => resp.handle(self).await,
            Response::Dialog(resp) => resp.handle(self).await,
            Response::Map(resp) => resp.handle(self),
            Response::SidePanel(resp) => resp.handle(self, term).await,
        }
    }

    fn handle_snapshot(&mut self, snapshot: Arc<WorldSnapshot>) {
        if self.paused {
            return;
        }

        self.snapshot = snapshot;

        if let Some(bot) = &mut self.bot {
            let exists = self.snapshot.bots().by_id(bot.id).is_some();

            bot.exists |= exists;

            if bot.exists && !exists {
                self.bot = None;
            }
        }
    }

    async fn handle_event(
        &mut self,
        event: DriverEvent,
    ) -> Result<Option<SnapshotStream>> {
        match event {
            DriverEvent::Join(handle) => {
                let mut snapshots = handle.snapshots();

                self.snapshot = snapshots.next_or_err().await?;
                self.camera = self.snapshot.map().center();
                self.handle = Some(handle);

                return Ok(Some(snapshots));
            }

            DriverEvent::Pause => {
                self.pause(true).await?;
            }

            DriverEvent::Resume => {
                self.pause(false).await?;
            }

            DriverEvent::SetPerms(perms) => {
                self.perms = perms;
            }

            DriverEvent::UpdatePerms(f) => {
                f(&mut self.perms);
            }

            DriverEvent::OpenDialog(dialog) => {
                self.dialog = Some(Dialog::Custom(dialog));
            }

            DriverEvent::CloseDialog => {
                self.dialog = None;
            }

            DriverEvent::SetHelp(help) => {
                self.help = help;
            }

            DriverEvent::SetStatus(status) => {
                self.status = status;
            }

            DriverEvent::Poll(f) => {
                self.poll = Some(f);
            }
        }

        Ok(None)
    }

    fn poll(&mut self) {
        if let Some(poll) = &mut self.poll {
            let ctxt = PollCtxt {
                world: &self.snapshot,
            };

            if poll(ctxt).is_ready() {
                self.poll = None;
            }
        }
    }

    async fn pause(&mut self, paused: bool) -> Result<()> {
        self.paused = paused;

        if self.perms.sync_pause_mode {
            if let Some(handle) = &self.handle {
                handle.pause(self.paused).await?;
            }
        }

        Ok(())
    }

    fn join_bot(&mut self, id: BotId) {
        self.bot = Some(JoinedBot {
            id,
            exists: false,
            is_followed: true,
        });
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

        let id = self.handle.as_ref().unwrap().create_bot(src, None).await;

        let id = match id {
            Ok(id) => id,

            Err(err) => {
                self.dialog =
                    Some(Dialog::Error(ErrorDialog::new(format!("{:?}", err))));

                return Ok(());
            }
        };

        self.join_bot(id);
        self.paused = false;

        Ok(())
    }
}

#[derive(Debug)]
struct JoinedBot {
    id: BotId,
    exists: bool,
    is_followed: bool,
}

#[derive(Debug)]
enum Response {
    BottomPanel(BottomPanelResponse),
    Dialog(DialogResponse),
    Map(MapResponse),
    SidePanel(SidePanelResponse),
}

#[derive(Debug)]
pub struct PollCtxt<'a> {
    pub world: &'a WorldSnapshot,
}

pub type PollFn = Box<dyn FnMut(PollCtxt) -> Poll<()> + Send + Sync>;

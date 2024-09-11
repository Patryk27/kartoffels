mod bottom;
mod dialog;
mod driver;
mod event;
mod map;
mod perms;
mod side;

use self::bottom::*;
use self::dialog::*;
pub use self::dialog::{HelpDialog, HelpDialogRef, HelpDialogResponse};
use self::event::*;
use self::map::*;
pub use self::perms::*;
use self::side::*;
use crate::DriverEventRx;
use anyhow::Result;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use futures_util::{stream, FutureExt};
use glam::IVec2;
use itertools::Either;
use kartoffels_ui::{Clear, Term, Ui};
use kartoffels_world::prelude::{
    BotId, BoxedSnapshotStream, Handle as WorldHandle,
    Snapshot as WorldSnapshot, SnapshotStreamExt,
};
use ratatui::layout::{Constraint, Layout};
use std::mem;
use std::ops::ControlFlow;
use std::sync::Arc;
use std::task::Poll;
use tokio::select;

pub async fn run(term: &mut Term, mut driver: DriverEventRx) -> Result<()> {
    let mut state = State::default();

    loop {
        let event = term
            .draw(|ui| {
                state.render(ui);
                ui.catch::<Event>()
            })
            .await?
            .flatten();

        if let Some(event) = event {
            match event.handle(&mut state, term).await? {
                ControlFlow::Continue(_) => {
                    continue;
                }
                ControlFlow::Break(_) => {
                    return Ok(());
                }
            }
        }

        state.poll(term, &mut driver).await?;
    }
}

struct State {
    bot: Option<JoinedBot>,
    camera: IVec2,
    dialog: Option<Dialog>,
    handle: Option<WorldHandle>,
    help: Option<HelpDialogRef>,
    map: Map,
    pause: PauseState,
    perms: Permissions,
    poll: Option<PollFn>,
    snapshot: Arc<WorldSnapshot>,
    snapshots: BoxedSnapshotStream,
    status: Option<String>,
}

impl State {
    fn render(&mut self, ui: &mut Ui) {
        // TODO doesn't belong here
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

        Clear::render(ui);

        ui.enable(self.dialog.is_none(), |ui| {
            ui.clamp(bottom_area, |ui| {
                BottomPanel::render(ui, self);
            });

            ui.clamp(side_area, |ui| {
                SidePanel::render(ui, self);
            });

            ui.clamp(map_area, |ui| {
                Map::render(ui, self);
            });
        });

        if let Some(dialog) = &mut self.dialog {
            dialog.render(ui, &self.snapshot);
        }
    }

    async fn poll(
        &mut self,
        term: &mut Term,
        driver: &mut DriverEventRx,
    ) -> Result<()> {
        let event = select! {
            snapshot = self.snapshots.next_or_err() => {
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
                self.set_snapshot(snapshot);
            }
            Some(Either::Right(event)) => {
                event.handle(self, term).await?;
            }
            None => {
                //
            }
        }

        // Opportunistically handle all pending driver events, to avoid
        // redrawing UI in-between them.
        //
        // This exists not as an optimization, but rather to prevent unnecessary
        // UI flashes e.g. when the driver closes and opens dialogs.
        while let Some(event) = driver.recv().now_or_never().flatten() {
            event.handle(self, term).await?;
        }

        if let Some(poll) = &mut self.poll {
            let ctxt = PollCtxt {
                world: &self.snapshot,
            };

            if poll(ctxt).is_ready() {
                self.poll = None;
            }
        }

        Ok(())
    }

    fn set_snapshot(&mut self, snapshot: Arc<WorldSnapshot>) {
        // If map size's changed, re-center the camera; this comes handy during
        // tutorial where the driver changes maps.
        if snapshot.map().size() != self.snapshot.map().size() {
            self.camera = snapshot.map().center();
        }

        self.snapshot = snapshot;

        if let Some(bot) = &mut self.bot {
            let exists_now = self.snapshot.bots().by_id(bot.id).is_some();

            bot.is_known_to_exist |= exists_now;

            if bot.is_known_to_exist && !exists_now {
                self.bot = None;
            }
        }
    }

    async fn pause(&mut self) -> Result<()> {
        match self.pause {
            PauseState::Resumed => {
                self.pause = PauseState::Paused(mem::replace(
                    &mut self.snapshots,
                    Box::new(stream::pending()),
                ));

                if self.perms.sync_pause
                    && let Some(handle) = &self.handle
                {
                    handle.pause().await?;
                }
            }

            PauseState::Paused(_) => {
                //
            }
        }

        Ok(())
    }

    async fn resume(&mut self) -> Result<()> {
        match mem::take(&mut self.pause) {
            PauseState::Resumed => {
                //
            }

            PauseState::Paused(snapshots) => {
                self.snapshots = snapshots;

                if self.perms.sync_pause
                    && let Some(handle) = &self.handle
                {
                    handle.resume().await?;
                }
            }
        }

        Ok(())
    }

    fn join_bot(&mut self, id: BotId) {
        self.bot = Some(JoinedBot {
            id,
            is_followed: true,
            is_known_to_exist: false,
        });

        self.map.blink_active = true;
        self.map.blink_interval.reset();
    }

    async fn upload_bot(&mut self, src: Either<String, Vec<u8>>) -> Result<()> {
        let src = match src {
            Either::Left(src) => {
                let src = src.trim().replace('\n', "");

                match BASE64_STANDARD.decode(src) {
                    Ok(src) => src,
                    Err(err) => {
                        self.dialog =
                            Some(Dialog::Error(ErrorDialog::new(format!(
                                "couldn't decode pasted content:\n\n{}",
                                err
                            ))));

                        return Ok(());
                    }
                }
            }

            Either::Right(src) => src,
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
        self.resume().await?;

        Ok(())
    }

    fn is_paused(&self) -> bool {
        matches!(self.pause, PauseState::Paused(_))
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            bot: Default::default(),
            camera: Default::default(),
            dialog: Default::default(),
            handle: Default::default(),
            help: Default::default(),
            map: Default::default(),
            pause: Default::default(),
            perms: Default::default(),
            poll: Default::default(),
            snapshot: Default::default(),
            snapshots: Box::new(stream::pending()),
            status: Default::default(),
        }
    }
}

#[derive(Default)]
enum PauseState {
    #[default]
    Resumed,
    Paused(BoxedSnapshotStream),
}

#[derive(Debug)]
struct JoinedBot {
    id: BotId,
    is_followed: bool,
    is_known_to_exist: bool,
}

#[derive(Debug)]
pub struct PollCtxt<'a> {
    pub world: &'a WorldSnapshot,
}

pub type PollFn = Box<dyn FnMut(PollCtxt) -> Poll<()> + Send>;

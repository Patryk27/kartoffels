mod bottom;
mod camera;
mod config;
mod ctrl;
mod event;
mod map;
mod modal;
mod overlay;
mod side;

use self::bottom::*;
use self::camera::*;
pub use self::config::*;
pub use self::ctrl::*;
use self::event::*;
use self::map::*;
use self::modal::*;
pub use self::modal::{HelpMsg, HelpMsgEvent, HelpMsgRef};
use self::overlay::*;
use self::side::*;
use crate::{theme, Clear, Fade, FadeDir, Frame, Ui, UiWidget};
use anyhow::Result;
use futures_util::FutureExt;
use glam::{IVec2, UVec2};
use kartoffels_store::{Session, Store, World};
use kartoffels_world::prelude as w;
use ratatui::layout::{Constraint, Layout};
use std::future::Future;
use std::ops::ControlFlow;
use std::sync::Arc;
use std::time::Instant;
use tokio::select;
use tokio::sync::oneshot;
use tracing::debug;

pub async fn run<CtrlFn, CtrlFut, CtrlOut>(
    store: &Store,
    sess: &Session,
    frame: &mut Frame,
    ctrl: CtrlFn,
) -> Result<Option<CtrlOut>>
where
    CtrlFn: FnOnce(GameCtrl) -> CtrlFut,
    CtrlFut: Future<Output = Result<CtrlOut>>,
{
    let (tx, rx) = GameCtrl::new();
    let view = run_once(store, sess, frame, rx);
    let ctrl = ctrl(tx);

    select! {
        result = view => result.map(|_| None),
        result = ctrl => result.map(Some),
    }
}

async fn run_once(
    store: &Store,
    sess: &Session,
    frame: &mut Frame,
    mut ctrl: GameCtrlRx,
) -> Result<()> {
    debug!("run()");

    let mut fade = Some(Fade::new(FadeDir::In));
    let mut view = View::default();

    loop {
        let event = frame
            .render(|ui| {
                view.tick(store);
                view.render(ui, sess, store);

                if let Some(fade) = &fade {
                    _ = fade.render(ui);
                }
            })
            .await?;

        if let Some(event) = event
            && let ControlFlow::Break(_) =
                event.handle(frame, &mut view).await?
        {
            fade = Some(Fade::new(FadeDir::Out));
        }

        view.poll(frame, &mut ctrl).await?;

        if let Some(fade) = &fade
            && fade.dir() == FadeDir::Out
            && fade.is_ready()
        {
            return Ok(());
        }
    }
}

#[derive(Default)]
struct View {
    bot: Option<JoinedBot>,
    camera: Camera,
    config: Config,
    events: Option<w::EventStream>,
    help: Option<HelpMsgRef>,
    label: Option<(String, Instant)>,
    map: Map,
    modal: Option<Box<Modal>>,
    mode: Mode,
    restart: Option<oneshot::Sender<()>>,
    snapshot: Arc<w::Snapshot>,
    snapshots: Option<w::SnapshotStream>,
    status: Status,
    world: Option<World>,
}

impl View {
    fn tick(&mut self, store: &Store) {
        // If we're following a bot, adjust the camera to the bot's current
        // position - unless we're under test, in which case we don't want to
        // move the camera since that makes tests less reproducible.
        if let Some(bot) = &self.bot
            && bot.follow
            && let Some(bot) = self.snapshot.bots.alive.get(bot.id)
            && !store.testing()
        {
            self.camera.look_at(bot.pos);
        }
    }

    fn render(&mut self, ui: &mut Ui<Event>, sess: &Session, store: &Store) {
        let [main_area, bottom_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)])
                .areas(ui.area);

        let [map_area, side_area] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(SidePanel::WIDTH),
        ])
        .areas(main_area);

        // TODO extract it somewhere else - it's a bit awkward, since we need to
        //      know `map_area` in order to calculate world coordinates
        if let Mode::SpawningBot {
            source,
            cursor_screen,
            cursor_world,
            cursor_valid,
            ..
        } = &mut self.mode
        {
            if let Some(pos) = ui.mouse_pos() {
                *cursor_screen = Some(pos);

                *cursor_world =
                    Some(self.camera.screen_to_world(pos, map_area));
            }

            if let Some(pos) = cursor_world {
                *cursor_valid = self.snapshot.tiles.get(*pos).is_floor();

                if ui.mouse_pressed() && *cursor_valid {
                    ui.throw(Event::CreateBot {
                        src: source.clone(),
                        pos: Some(*pos),
                        follow: false,
                    });
                }
            }
        }

        ui.add(Clear);

        for area in [side_area, bottom_area] {
            for y in area.top()..area.bottom() {
                for x in area.left()..area.right() {
                    ui.buf[(x, y)].set_bg(theme::DARKER_GRAY);
                }
            }
        }

        ui.enabled(self.modal.is_none(), |ui| {
            ui.at(bottom_area, |ui| {
                BottomPanel::render(ui, self);
            });

            if self.world.is_some() {
                ui.enabled(self.config.enabled, |ui| {
                    ui.at(side_area, |ui| {
                        SidePanel::render(ui, self);
                    });

                    ui.at(map_area, |ui| {
                        self.map.render(ui, self);
                    });
                });
            }

            Overlay::render(ui, store, self);
        });

        if let Some(modal) = &mut self.modal {
            modal.render(ui, sess, &self.snapshot);
        }
    }

    async fn poll(
        &mut self,
        frame: &mut Frame,
        ctrl: &mut GameCtrlRx,
    ) -> Result<()> {
        while let Some(event) = ctrl.recv().now_or_never().flatten() {
            event.handle(self, frame).await?;
        }

        if let Some(events) = &mut self.events
            && let Some(Ok(event)) = events.next().now_or_never()
            && let w::Event::BotReachedBreakpoint { id } = event.event
        {
            self.handle_breakpoint(id).await?;
        }

        if let Some(snapshots) = &mut self.snapshots
            && let Some(snapshot) = snapshots.next().now_or_never()
        {
            self.refresh(snapshot?);
        }

        Ok(())
    }

    async fn handle_breakpoint(&mut self, id: w::BotId) -> Result<()> {
        debug!("got a breakpoint");

        self.status = Status::Paused {
            on_breakpoint: Some(Instant::now()),
        };

        if let Some(snapshots) = &mut self.snapshots
            && let Some(bot) = snapshots.next().await?.bots.alive.get(id)
        {
            self.join(id, true);
            self.camera.look_at(bot.pos);
        }

        Ok(())
    }

    fn refresh(&mut self, snapshot: Arc<w::Snapshot>) {
        debug!(version=?snapshot.version, "refreshing");

        // If the map's size has changed, recenter the camera.
        //
        // This comes handy for controllers which call `world.set_map()`, e.g.
        // the tutorial.
        if snapshot.tiles.size() != self.snapshot.tiles.size() {
            self.camera.look_at(snapshot.tiles.center());
        }

        // If the bot we're tracking has disappeared, disconnect from it.
        //
        // This comes handy for controllers which manually modify the bots, e.g.
        // the tutorial.
        if let Some(bot) = &mut self.bot {
            let exists = snapshot.bots.has(bot.id);

            match bot.exists {
                Some(true) => {
                    if !exists {
                        self.bot = None;
                    }
                }
                _ => {
                    bot.exists = Some(exists);
                }
            }
        }

        self.snapshot = snapshot;
    }

    async fn pause(&mut self) -> Result<()> {
        if self.status.is_active() {
            debug!("pausing");

            self.status = Status::Paused {
                on_breakpoint: None,
            };

            self.snapshots = None;

            if self.config.sync_pause {
                if let Some(world) = &self.world {
                    world.pause().await?;
                } else {
                    debug!("suspicious: got no world handle");
                }
            }
        }

        Ok(())
    }

    async fn resume(&mut self) -> Result<()> {
        if self.status.is_paused() {
            debug!("resuming");

            self.status = Status::Active;

            self.snapshots =
                self.world.as_ref().map(|handle| handle.snapshots());

            if self.config.sync_pause {
                if let Some(world) = &self.world {
                    world.resume().await?;
                } else {
                    debug!("suspicious: got no world handle");
                }
            }
        }

        Ok(())
    }

    fn join(&mut self, id: w::BotId, follow: bool) {
        debug!(?id, "joining");

        self.bot = Some(JoinedBot {
            id,
            follow,
            exists: None,
        });

        self.map.blink = Instant::now();
    }
}

#[derive(Clone, Debug, Default)]
enum Mode {
    #[default]
    Default,

    SpawningBot {
        source: Vec<u8>,
        cursor_screen: Option<UVec2>,
        cursor_world: Option<IVec2>,
        cursor_valid: bool,
    },
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum Status {
    #[default]
    Active,

    Paused {
        // None -> game has been paused normally
        // Some(...) -> game has been paused because of a bot's breakpoint
        on_breakpoint: Option<Instant>,
    },
}

impl Status {
    fn is_active(&self) -> bool {
        matches!(self, Self::Active)
    }

    fn is_paused(&self) -> bool {
        matches!(self, Self::Paused { .. })
    }
}

#[derive(Clone, Debug)]
struct JoinedBot {
    id: w::BotId,
    follow: bool,
    exists: Option<bool>,
}

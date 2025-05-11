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
use kartoffels_world::prelude::{
    BotId, Snapshot as WorldSnapshot, SnapshotStream,
};
use ratatui::layout::{Constraint, Layout};
use std::future::Future;
use std::ops::ControlFlow;
use std::sync::Arc;
use std::time::Instant;
use tokio::select;
use tokio::sync::oneshot;
use tracing::debug;

pub async fn run<CtrlFn, CtrlFut>(
    store: &Store,
    sess: &Session,
    frame: &mut Frame,
    ctrl: CtrlFn,
) -> Result<()>
where
    CtrlFn: FnOnce(GameCtrl) -> CtrlFut,
    CtrlFut: Future<Output = Result<()>>,
{
    let (tx, rx) = GameCtrl::new();
    let view = run_once(store, sess, frame, rx);
    let ctrl = ctrl(tx);

    select! {
        result = view => result,
        result = ctrl => result,
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
    let mut tick = Instant::now();
    let mut state = State::default();

    loop {
        let event = frame
            .tick(|ui| {
                state.tick(tick.elapsed().as_secs_f32(), store);
                state.render(ui, sess, store);

                if let Some(fade) = &fade {
                    _ = fade.render(ui);
                }

                tick = Instant::now();
            })
            .await?;

        if let Some(event) = event
            && let ControlFlow::Break(_) =
                event.handle(frame, &mut state).await?
        {
            fade = Some(Fade::new(FadeDir::Out));
        }

        state.poll(frame, &mut ctrl).await?;

        if let Some(fade) = &fade
            && fade.dir() == FadeDir::Out
            && fade.is_ready()
        {
            return Ok(());
        }
    }
}

#[derive(Default)]
struct State {
    bot: Option<JoinedBot>,
    camera: Camera,
    config: Config,
    help: Option<HelpMsgRef>,
    label: Option<(String, Instant)>,
    map: Map,
    modal: Option<Box<Modal>>,
    mode: Mode,
    paused: bool,
    restart: Option<oneshot::Sender<()>>,
    snapshot: Arc<WorldSnapshot>,
    snapshots: Option<SnapshotStream>,
    world: Option<World>,
}

impl State {
    fn tick(&mut self, dt: f32, store: &Store) {
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

        self.camera.tick(dt, store);
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

        ui.enable(self.modal.is_none(), |ui| {
            ui.clamp(bottom_area, |ui| {
                BottomPanel::render(ui, self);
            });

            if self.world.is_some() {
                ui.enable(self.config.enabled, |ui| {
                    ui.clamp(side_area, |ui| {
                        SidePanel::render(ui, self);
                    });

                    ui.clamp(map_area, |ui| {
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

        if let Some(snapshots) = &mut self.snapshots
            && let Some(snapshot) = snapshots.next().now_or_never()
        {
            self.update_snapshot(snapshot?);
        }

        Ok(())
    }

    fn update_snapshot(&mut self, snapshot: Arc<WorldSnapshot>) {
        // If map size's changed, recenter the camera - this comes handy for
        // controllers which call `world.set_map()`, e.g. the tutorial
        if snapshot.tiles.size() != self.snapshot.tiles.size() {
            self.camera.set(snapshot.tiles.center());
        }

        self.snapshot = snapshot;

        if let Some(bot) = &mut self.bot {
            let exists_now = self.snapshot.bots.has(bot.id);

            bot.exists |= exists_now;

            if bot.exists && !exists_now {
                self.bot = None;
            }
        }
    }

    async fn pause(&mut self) -> Result<()> {
        if !self.paused {
            self.paused = true;
            self.snapshots = None;

            if self.config.sync_pause
                && let Some(handle) = &self.world
            {
                handle.pause().await?;
            }
        }

        Ok(())
    }

    async fn resume(&mut self) -> Result<()> {
        if self.paused {
            self.paused = false;

            self.snapshots =
                self.world.as_ref().map(|handle| handle.snapshots());

            if self.config.sync_pause
                && let Some(handle) = &self.world
            {
                handle.resume().await?;
            }
        }

        Ok(())
    }

    fn join_bot(&mut self, id: BotId, follow: bool) {
        self.bot = Some(JoinedBot {
            id,
            follow,
            exists: false,
        });

        self.map.blink = Instant::now();
    }
}

#[derive(Debug, Default)]
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

#[derive(Debug)]
struct JoinedBot {
    id: BotId,
    follow: bool,
    exists: bool,
}

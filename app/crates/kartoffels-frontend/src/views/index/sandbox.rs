mod ctrl;
mod sandbox_size;
mod sandbox_theme;

use self::sandbox_size::*;
use self::sandbox_theme::*;
use crate::views::game;
use crate::Background;
use anyhow::Result;
use glam::uvec2;
use kartoffels_store::{SessionId, Store};
use kartoffels_ui::{Button, Fade, FadeDir, KeyCode, Term, Ui, UiWidget};
use kartoffels_world::prelude::{ArenaTheme, DungeonTheme, Theme};
use std::ops::ControlFlow;
use tracing::debug;

pub async fn run(
    store: &Store,
    sess: SessionId,
    term: &mut Term,
    bg: &Background,
) -> Result<()> {
    let mut fade_in = false;

    // We keep those outside `run_once()`, because we want to preserve settings
    // when user goes back to the form, for convenience
    let mut size = SandboxSize::Medium;
    let mut theme = SandboxTheme::Dungeon;

    loop {
        if let Some(theme) =
            run_once(store, term, bg, fade_in, &mut size, &mut theme).await?
        {
            game::run(store, sess, term, |game| ctrl::run(store, theme, game))
                .await?;

            fade_in = true;
        } else {
            return Ok(());
        }
    }
}

async fn run_once(
    store: &Store,
    term: &mut Term,
    bg: &Background,
    fade_in: bool,
    size: &mut SandboxSize,
    theme: &mut SandboxTheme,
) -> Result<Option<Theme>> {
    debug!("run()");

    let mut view = {
        let fade_in = if fade_in && !store.testing() {
            Some(Fade::new(FadeDir::In))
        } else {
            None
        };

        View {
            bg,
            fade_in,
            fade_out: None,
            form: Form {
                focus: None,
                size,
                theme,
            },
        }
    };

    loop {
        let event = term
            .frame(|ui| {
                view.render(ui);
            })
            .await?;

        if let Some((fade, theme)) = &view.fade_out {
            if fade.is_completed() {
                return Ok(Some(theme.clone()));
            }

            continue;
        }

        if let Some(event) = event
            && let ControlFlow::Break(_) = view.handle(event)
        {
            return Ok(None);
        }
    }
}

#[derive(Debug)]
struct View<'a> {
    bg: &'a Background,
    fade_in: Option<Fade>,
    fade_out: Option<(Fade, Theme)>,
    form: Form<'a>,
}

impl View<'_> {
    fn render(&mut self, ui: &mut Ui<Event>) {
        self.bg.render(ui);
        self.form.render(ui);

        if let Some(fade) = &self.fade_in
            && fade.render(ui).is_completed()
        {
            self.fade_in = None;
        }

        if let Some((fade, _)) = &self.fade_out {
            fade.render(ui);
        }
    }

    fn handle(&mut self, event: Event) -> ControlFlow<()> {
        match event {
            Event::GoBack => {
                if self.form.focus.is_some() {
                    self.form.focus = None;
                } else {
                    return ControlFlow::Break(());
                }
            }

            Event::Confirm => {
                let fade = Fade::new(FadeDir::Out);
                let theme = self.form.confirm();

                self.fade_out = Some((fade, theme));
            }

            Event::FocusOn(val) => {
                self.form.focus = val;
            }

            Event::SetSize(val) => {
                *self.form.size = val;
                self.form.focus = None;
            }

            Event::SetTheme(val) => {
                *self.form.theme = val;
                self.form.focus = None;
            }
        }

        ControlFlow::Continue(())
    }
}

#[derive(Debug)]
struct Form<'a> {
    focus: Option<Focus>,
    size: &'a mut SandboxSize,
    theme: &'a mut SandboxTheme,
}

impl Form<'_> {
    fn render(&mut self, ui: &mut Ui<Event>) {
        let width = 40;
        let height = self.height();
        let title = self.title();

        ui.info_window(width, height, Some(title), |ui| {
            self.render_body(ui);
            self.render_footer(ui);
        });
    }

    fn title(&self) -> &'static str {
        match &self.focus {
            Some(Focus::SandboxSize) => " sandbox › choose-size ",
            Some(Focus::SandboxTheme) => " sandbox › choose-theme ",
            None => " sandbox ",
        }
    }

    fn height(&self) -> u16 {
        match &self.focus {
            Some(Focus::SandboxSize) => SandboxSize::height() + 2,
            Some(Focus::SandboxTheme) => SandboxTheme::height() + 2,
            None => 4,
        }
    }

    fn render_body(&self, ui: &mut Ui<Event>) {
        match &self.focus {
            Some(Focus::SandboxSize) => {
                SandboxSize::render_choice(ui);
            }
            Some(Focus::SandboxTheme) => {
                SandboxTheme::render_choice(ui);
            }
            None => {
                SandboxSize::render_focus(ui, self.size);
                SandboxTheme::render_focus(ui, self.theme);
            }
        }
    }

    fn render_footer(&self, ui: &mut Ui<Event>) {
        ui.space(1);

        ui.row(|ui| {
            Button::new(KeyCode::Escape, "go-back")
                .throwing(Event::GoBack)
                .render(ui);

            if self.focus.is_none() {
                Button::new(KeyCode::Enter, "create")
                    .right_aligned()
                    .throwing(Event::Confirm)
                    .render(ui);
            }
        });
    }

    fn confirm(&self) -> Theme {
        match &self.theme {
            SandboxTheme::Arena => {
                let radius = match &self.size {
                    SandboxSize::Tiny => 4,
                    SandboxSize::Small => 8,
                    SandboxSize::Medium => 16,
                    SandboxSize::Large => 24,
                };

                Theme::Arena(ArenaTheme::new(radius))
            }

            SandboxTheme::Dungeon => {
                let size = match &self.size {
                    SandboxSize::Tiny => uvec2(10, 10),
                    SandboxSize::Small => uvec2(20, 15),
                    SandboxSize::Medium => uvec2(60, 30),
                    SandboxSize::Large => uvec2(80, 50),
                };

                Theme::Dungeon(DungeonTheme::new(size))
            }
        }
    }
}

#[derive(Debug)]
enum Event {
    GoBack,
    Confirm,
    FocusOn(Option<Focus>),
    SetSize(SandboxSize),
    SetTheme(SandboxTheme),
}

#[derive(Debug, PartialEq, Eq)]
enum Focus {
    SandboxSize,
    SandboxTheme,
}

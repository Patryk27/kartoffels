mod ctrl;

use crate::views::game;
use crate::Background;
use anyhow::Result;
use glam::uvec2;
use kartoffels_store::{SessionId, Store};
use kartoffels_ui::{Button, Fade, FadeDir, Render, Term, Ui};
use kartoffels_world::prelude::{ArenaTheme, DungeonTheme, Theme};
use std::fmt;
use termwiz::input::KeyCode;
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

    let mut fade_in = if fade_in && !store.testing() {
        Some(Fade::new(FadeDir::In))
    } else {
        None
    };

    let mut fade_out: Option<(Fade, Theme)> = None;
    let mut focus = None;

    loop {
        let event = term
            .draw(|ui| {
                let width = 40;

                let height = match focus {
                    Some(Focus::Size) => 4 + SandboxSize::all().count(),
                    Some(Focus::Theme) => 4 + SandboxTheme::all().count(),
                    None => 4,
                } as u16;

                bg.render(ui);

                ui.info_window(width, height, Some(" sandbox "), |ui| {
                    render_form(ui, focus.as_ref(), theme, size);
                    render_footer(ui, focus.as_ref());
                });

                if let Some(fade) = &fade_in {
                    if fade.render(ui).is_completed() {
                        fade_in = None;
                    }
                }

                if let Some((fade, _)) = &fade_out {
                    fade.render(ui);
                }
            })
            .await?;

        term.poll().await?;

        if let Some((fade, theme)) = &fade_out {
            if fade.is_completed() {
                return Ok(Some(theme.clone()));
            }

            continue;
        }

        if let Some(event) = event {
            match event {
                Event::GoBack => {
                    if focus.is_some() {
                        focus = None;
                    } else {
                        return Ok(None);
                    }
                }

                Event::CreateSandbox => {
                    let fade = Fade::new(FadeDir::Out);
                    let theme = create_theme(size, theme);

                    fade_out = Some((fade, theme));
                }

                Event::SetFocus(val) => {
                    focus = val;
                }

                Event::SetSize(val) => {
                    *size = val;
                    focus = None;
                }

                Event::SetTheme(val) => {
                    *theme = val;
                    focus = None;
                }
            }
        }
    }
}

fn render_form(
    ui: &mut Ui<Event>,
    focus: Option<&Focus>,
    theme: &SandboxTheme,
    size: &SandboxSize,
) {
    match focus {
        Some(Focus::Size) => {
            ui.line("choose size:");
            ui.space(1);

            for size in SandboxSize::all() {
                Button::new(KeyCode::Char(size.key()), size.to_string())
                    .throwing(Event::SetSize(size))
                    .render(ui);
            }
        }

        Some(Focus::Theme) => {
            ui.line("choose theme:");
            ui.space(1);

            for ty in SandboxTheme::all() {
                Button::new(KeyCode::Char(ty.key()), ty.to_string())
                    .throwing(Event::SetTheme(ty))
                    .render(ui);
            }
        }

        None => {
            Button::new(KeyCode::Char('s'), format!("size: {size}"))
                .throwing(Event::SetFocus(Some(Focus::Size)))
                .render(ui);

            Button::new(KeyCode::Char('t'), format!("theme: {theme}"))
                .throwing(Event::SetFocus(Some(Focus::Theme)))
                .render(ui);
        }
    }
}

fn render_footer(ui: &mut Ui<Event>, focus: Option<&Focus>) {
    ui.space(1);

    ui.row(|ui| {
        Button::new(KeyCode::Escape, "go back")
            .throwing(Event::GoBack)
            .render(ui);

        if focus.is_none() {
            Button::new(KeyCode::Enter, "create sandbox")
                .right_aligned()
                .throwing(Event::CreateSandbox)
                .render(ui);
        }
    });
}

fn create_theme(size: &SandboxSize, theme: &SandboxTheme) -> Theme {
    match theme {
        SandboxTheme::Arena => {
            let radius = match size {
                SandboxSize::Tiny => 4,
                SandboxSize::Small => 8,
                SandboxSize::Medium => 16,
                SandboxSize::Large => 24,
            };

            Theme::Arena(ArenaTheme::new(radius))
        }

        SandboxTheme::Dungeon => {
            let size = match size {
                SandboxSize::Tiny => uvec2(10, 10),
                SandboxSize::Small => uvec2(20, 15),
                SandboxSize::Medium => uvec2(60, 30),
                SandboxSize::Large => uvec2(80, 50),
            };

            Theme::Dungeon(DungeonTheme::new(size))
        }
    }
}

#[derive(Debug)]
enum Event {
    GoBack,
    CreateSandbox,
    SetFocus(Option<Focus>),
    SetSize(SandboxSize),
    SetTheme(SandboxTheme),
}

#[derive(Debug, PartialEq, Eq)]
enum Focus {
    Size,
    Theme,
}

#[derive(Clone, Debug)]
enum SandboxSize {
    Tiny,
    Small,
    Medium,
    Large,
}

impl SandboxSize {
    fn all() -> impl Iterator<Item = Self> {
        [Self::Tiny, Self::Small, Self::Medium, Self::Large].into_iter()
    }

    fn key(&self) -> char {
        match self {
            Self::Tiny => 't',
            Self::Small => 's',
            Self::Medium => 'm',
            Self::Large => 'l',
        }
    }
}

impl fmt::Display for SandboxSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Tiny => "tiny",
                Self::Small => "small",
                Self::Medium => "medium",
                Self::Large => "large",
            }
        )
    }
}

#[derive(Clone, Debug)]
enum SandboxTheme {
    Arena,
    Dungeon,
}

impl SandboxTheme {
    fn all() -> impl Iterator<Item = Self> {
        [Self::Arena, Self::Dungeon].into_iter()
    }
}

impl SandboxTheme {
    fn key(&self) -> char {
        match self {
            Self::Arena => 'a',
            Self::Dungeon => 'd',
        }
    }
}

impl fmt::Display for SandboxTheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Arena => "arena",
                Self::Dungeon => "dungeon",
            }
        )
    }
}

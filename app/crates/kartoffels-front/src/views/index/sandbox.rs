mod ctrl;
mod sandbox_size;
mod sandbox_theme;

use self::sandbox_size::*;
use self::sandbox_theme::*;
use super::WINDOW_WIDTH;
use crate::views::game;
use crate::{BgMap, Fade, Frame, Ui, UiWidget};
use anyhow::Result;
use glam::uvec2;
use kartoffels_store::{Session, Store};
use kartoffels_world::prelude as w;
use termwiz::input::KeyCode;
use tracing::debug;

pub async fn run(
    store: &Store,
    sess: &Session,
    frame: &mut Frame,
    bg: &BgMap,
) -> Result<()> {
    let mut fade_in = false;

    // We keep those outside `main()`, because we want to preserve settings
    // when user goes back to the view, for convenience
    let mut size = SandboxSize::Medium;
    let mut theme = SandboxTheme::Cave;

    loop {
        debug!("run()");

        if let Some(theme) =
            main(store, frame, bg, fade_in, &mut size, &mut theme).await?
        {
            game::run(store, sess, frame, |game| ctrl::run(store, theme, game))
                .await?;

            fade_in = true;
        } else {
            return Ok(());
        }
    }
}

async fn main(
    store: &Store,
    frame: &mut Frame,
    bg: &BgMap,
    fade_in: bool,
    size: &mut SandboxSize,
    theme: &mut SandboxTheme,
) -> Result<Option<w::Theme>> {
    let mut fade = Fade::new(store, fade_in);

    let mut view = View {
        size,
        theme,
        focus: None,
    };

    loop {
        let event = frame
            .render(|ui| {
                bg.render(ui);
                view.render(ui);
                fade.render(ui);
            })
            .await?;

        if let Some(event) = event {
            match event {
                Event::GoBack => {
                    if view.focus.is_some() {
                        view.focus = None;
                    } else {
                        return Ok(None);
                    }
                }

                Event::Confirm => {
                    fade.out(view.confirm());
                }

                Event::FocusOn(val) => {
                    view.focus = val;
                }

                Event::SetSize(val) => {
                    *view.size = val;
                    view.focus = None;
                }

                Event::SetTheme(val) => {
                    *view.theme = val;
                    view.focus = None;
                }
            }
        }

        if let Some(event) = fade.poll() {
            return Ok(Some(event));
        }
    }
}

#[derive(Debug)]
struct View<'a> {
    size: &'a mut SandboxSize,
    theme: &'a mut SandboxTheme,
    focus: Option<Focus>,
}

impl View<'_> {
    fn render(&mut self, ui: &mut Ui<Event>) {
        let width = WINDOW_WIDTH;
        let height = self.height();
        let title = self.title();

        ui.imodal(width, height, Some(title), |ui| {
            self.render_body(ui);
            self.render_footer(ui);
        });
    }

    fn title(&self) -> &'static str {
        match &self.focus {
            None => "sandbox",
            Some(Focus::SandboxSize) => "sandbox › choose-size",
            Some(Focus::SandboxTheme) => "sandbox › choose-theme",
        }
    }

    fn height(&self) -> u16 {
        match &self.focus {
            None => 4,
            Some(Focus::SandboxSize) => SandboxSize::height() + 2,
            Some(Focus::SandboxTheme) => SandboxTheme::height() + 2,
        }
    }

    fn render_body(&self, ui: &mut Ui<Event>) {
        match &self.focus {
            None => {
                SandboxSize::render_btn(ui, self.size);
                SandboxTheme::render_btn(ui, self.theme);
            }

            Some(Focus::SandboxSize) => {
                SandboxSize::render_form(ui);
            }
            Some(Focus::SandboxTheme) => {
                SandboxTheme::render_form(ui);
            }
        }
    }

    fn render_footer(&self, ui: &mut Ui<Event>) {
        ui.space(1);

        ui.row(|ui| {
            ui.btn("exit", KeyCode::Escape, |btn| btn.throwing(Event::GoBack));

            if self.focus.is_none() {
                ui.btn("create", KeyCode::Enter, |btn| {
                    btn.right_aligned().throwing(Event::Confirm)
                });
            }
        });
    }

    fn confirm(&self) -> w::Theme {
        match self.theme {
            SandboxTheme::Arena => {
                let radius = match self.size {
                    SandboxSize::Tiny => 4,
                    SandboxSize::Small => 8,
                    SandboxSize::Medium => 16,
                    SandboxSize::Large => 24,
                };

                w::Theme::Arena(w::ArenaTheme::new(radius))
            }

            SandboxTheme::Cave => {
                let size = match self.size {
                    SandboxSize::Tiny => uvec2(16, 8),
                    SandboxSize::Small => uvec2(24, 16),
                    SandboxSize::Medium => uvec2(64, 32),
                    SandboxSize::Large => uvec2(128, 64),
                };

                w::Theme::Cave(w::CaveTheme::new(size))
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

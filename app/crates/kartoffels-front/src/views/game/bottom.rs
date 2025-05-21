use super::{Event, Mode, Status, View};
use crate::{theme, Button, Ui, UiWidget};
use kartoffels_world::prelude as w;
use ratatui::prelude::Rect;
use ratatui::style::Stylize;
use ratatui::text::Span;
use termwiz::input::KeyCode;

#[derive(Debug)]
pub struct BottomPanel;

impl BottomPanel {
    pub fn render(ui: &mut Ui<Event>, view: &View) {
        Self::render_status(ui, view);

        ui.row(|ui| {
            Self::render_go_back_btn(ui);

            if view.restart.is_some() {
                Self::render_restart_btn(ui);
                return;
            }

            match &view.mode {
                Mode::Default => {
                    if view.world.is_some() {
                        ui.enabled(view.config.enabled, |ui| {
                            Self::render_status_btn(ui, view);
                            Self::render_help_btn(ui, view);
                            Self::render_bots_btn(ui, view);
                            Self::render_overclock_btn(ui, view);
                        });
                    }
                }

                Mode::SpawningBot { .. } => {
                    //
                }
            }
        });
    }

    fn render_go_back_btn(ui: &mut Ui<Event>) {
        ui.btn("exit", KeyCode::Escape, |btn| {
            btn.throwing(Event::GoBack { confirm: true })
        });
    }

    fn render_status_btn(ui: &mut Ui<Event>, view: &View) {
        ui.space(2);

        let label = if view.status.is_paused() {
            "resume"
        } else {
            "pause"
        };

        ui.enabled(view.config.can_pause, |ui| {
            ui.btn(label, KeyCode::Char(' '), |btn| {
                let btn = btn.throwing(Event::ToggleStatus);

                if let Status::Paused {
                    on_breakpoint: Some(tt),
                } = view.status
                {
                    if tt.elapsed().as_millis() % 1000 <= 500 {
                        btn.bold()
                    } else {
                        btn
                    }
                } else {
                    btn
                }
            });
        });
    }

    fn render_restart_btn(ui: &mut Ui<Event>) {
        ui.space(2);

        ui.btn("restart", KeyCode::Char('r'), |btn| {
            btn.throwing(Event::Restart)
        });
    }

    fn render_help_btn(ui: &mut Ui<Event>, view: &View) {
        ui.space(2);

        ui.enabled(view.help.is_some(), |ui| {
            ui.btn("help", KeyCode::Char('h'), |btn| {
                btn.throwing(Event::OpenHelpModal)
            });
        });
    }

    fn render_bots_btn(ui: &mut Ui<Event>, view: &View) {
        if !view.config.hero_mode {
            ui.space(2);

            ui.btn("bots", KeyCode::Char('b'), |btn| {
                btn.throwing(Event::OpenBotsModal)
            });
        }
    }

    fn render_overclock_btn(ui: &mut Ui<Event>, view: &View) {
        if view.config.can_overclock {
            ui.space(2);

            Button::multi("overclock")
                .throwing_on(
                    KeyCode::Char('1'),
                    Event::Overclock {
                        clock: w::Clock::Normal,
                    },
                )
                .throwing_on(
                    KeyCode::Char('2'),
                    Event::Overclock {
                        clock: w::Clock::Fast,
                    },
                )
                .throwing_on(
                    KeyCode::Char('3'),
                    Event::Overclock {
                        clock: w::Clock::Faster,
                    },
                )
                .render(ui);
        }
    }

    fn render_status(ui: &mut Ui<Event>, view: &View) {
        let Some(span) = Self::status(view) else {
            return;
        };

        let width = span.content.len() as u16;

        let area = Rect {
            x: ui.area.width - width,
            y: ui.area.y,
            width,
            height: 1,
        };

        ui.add_at(area, span);
    }

    fn status(view: &View) -> Option<Span> {
        if let Some((label, tt)) = &view.label {
            let span = Span::raw(label);
            let blink = tt.elapsed().as_millis() % 1000 <= 500;

            return Some(if blink {
                span.fg(theme::BG).bg(theme::YELLOW)
            } else {
                span.fg(theme::YELLOW)
            });
        }

        #[allow(clippy::question_mark)]
        if view.world.is_none() {
            return None;
        }

        if let Status::Paused {
            on_breakpoint: None,
        } = view.status
        {
            return Some(Span::raw("paused").fg(theme::FG).bg(theme::RED));
        }

        if let Status::Paused {
            on_breakpoint: Some(tt),
        } = view.status
        {
            let span = Span::raw("breakpoint");
            let blink = tt.elapsed().as_millis() % 1000 <= 500;

            return Some(if blink {
                span.fg(theme::FG).bg(theme::RED)
            } else {
                span.fg(theme::RED).bg(theme::FG)
            });
        }

        let speed = match view.snapshot.clock {
            w::Clock::Normal | w::Clock::Manual { .. } => return None,
            w::Clock::Fast => "spd:fast",
            w::Clock::Faster => "spd:faster",
            w::Clock::Unlimited => "spd:∞",
        };

        Some(Span::raw(speed).fg(theme::WASHED_PINK))
    }
}

use super::{Event, Mode, State};
use crate::{theme, Button, Ui, UiWidget};
use kartoffels_world::prelude::Clock;
use ratatui::prelude::Rect;
use ratatui::style::Stylize;
use ratatui::text::Span;
use termwiz::input::KeyCode;

#[derive(Debug)]
pub struct BottomPanel;

impl BottomPanel {
    pub fn render(ui: &mut Ui<Event>, state: &State) {
        if state.world.is_some() {
            Self::render_label(ui, state);
        }

        ui.row(|ui| {
            Self::render_go_back_btn(ui);

            if state.restart.is_some() {
                Self::render_restart_btn(ui);
                return;
            }

            match &state.mode {
                Mode::Default => {
                    if state.world.is_some() {
                        ui.enabled(state.config.enabled, |ui| {
                            Self::render_pause_btn(ui, state);
                            Self::render_help_btn(ui, state);
                            Self::render_bots_btn(ui, state);
                            Self::render_overclock_btn(ui, state);
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
        ui.btn("go-back", KeyCode::Escape, |btn| {
            btn.throwing(Event::GoBack { confirm: true })
        });
    }

    fn render_pause_btn(ui: &mut Ui<Event>, state: &State) {
        ui.space(2);

        let label = if state.paused { "resume" } else { "pause" };

        ui.enabled(state.config.can_pause, |ui| {
            ui.btn(label, KeyCode::Char(' '), |btn| {
                btn.throwing(Event::TogglePause)
            });
        });
    }

    fn render_restart_btn(ui: &mut Ui<Event>) {
        ui.space(2);

        ui.btn("restart", KeyCode::Char('r'), |btn| {
            btn.throwing(Event::Restart)
        });
    }

    fn render_help_btn(ui: &mut Ui<Event>, state: &State) {
        ui.space(2);

        ui.enabled(state.help.is_some(), |ui| {
            ui.btn("help", KeyCode::Char('h'), |btn| {
                btn.throwing(Event::OpenHelpModal)
            });
        });
    }

    fn render_bots_btn(ui: &mut Ui<Event>, state: &State) {
        if !state.config.hero_mode {
            ui.space(2);

            ui.btn("bots", KeyCode::Char('b'), |btn| {
                btn.throwing(Event::OpenBotsModal)
            });
        }
    }

    fn render_overclock_btn(ui: &mut Ui<Event>, state: &State) {
        if state.config.can_overclock {
            ui.space(2);

            Button::multi("overclock")
                .throwing_on(
                    KeyCode::Char('1'),
                    Event::Overclock {
                        clock: Clock::Normal,
                    },
                )
                .throwing_on(
                    KeyCode::Char('2'),
                    Event::Overclock { clock: Clock::Fast },
                )
                .throwing_on(
                    KeyCode::Char('3'),
                    Event::Overclock {
                        clock: Clock::Faster,
                    },
                )
                .render(ui);
        }
    }

    fn render_label(ui: &mut Ui<Event>, state: &State) {
        let span = if state.paused {
            Some(Span::raw("paused").fg(theme::FG).bg(theme::RED))
        } else if let Some((label, label_tt)) = &state.label {
            let span = Span::raw(label);

            if label_tt.elapsed().as_millis() % 1000 <= 500 {
                Some(span.fg(theme::BG).bg(theme::YELLOW))
            } else {
                Some(span.fg(theme::YELLOW))
            }
        } else {
            let speed = match state.snapshot.clock {
                Clock::Normal | Clock::Manual { .. } => None,
                Clock::Fast => Some("spd:fast"),
                Clock::Faster => Some("spd:faster"),
                Clock::Unlimited => Some("spd:∞"),
            };

            speed.map(|speed| Span::raw(speed).fg(theme::WASHED_PINK))
        };

        let Some(span) = span else {
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
}

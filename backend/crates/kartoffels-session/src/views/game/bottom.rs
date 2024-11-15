use super::{Event, Mode, State};
use kartoffels_store::Store;
use kartoffels_ui::{theme, Button, Render, Ui};
use kartoffels_world::prelude::ClockSpeed;
use ratatui::prelude::Rect;
use ratatui::style::Stylize;
use ratatui::text::Span;
use termwiz::input::KeyCode;

#[derive(Debug)]
pub struct BottomPanel;

impl BottomPanel {
    pub fn render(ui: &mut Ui<Event>, state: &State, store: &Store) {
        ui.row(|ui| {
            Self::render_go_back_btn(ui);

            if state.restart.is_some() {
                Self::render_restart_btn(ui);
                return;
            }

            match &state.mode {
                Mode::Default => {
                    if state.handle.is_some() {
                        ui.enable(state.config.enabled, |ui| {
                            Self::render_help_btn(ui, state);
                            Self::render_pause_btn(ui, state);
                            Self::render_bots_btn(ui, state);
                            Self::render_overclock_btn(ui, state);
                            Self::render_debug_btn(ui, store);
                        });
                    }
                }

                Mode::SpawningBot { .. } => {
                    //
                }
            }
        });

        if state.handle.is_some() {
            Self::render_status(ui, state);
        }
    }

    fn render_go_back_btn(ui: &mut Ui<Event>) {
        Button::new(KeyCode::Escape, "go-back")
            .throwing(Event::GoBack {
                needs_confirmation: true,
            })
            .render(ui);
    }

    fn render_restart_btn(ui: &mut Ui<Event>) {
        ui.space(2);

        Button::new(KeyCode::Char('r'), "restart")
            .throwing(Event::Restart)
            .render(ui);
    }

    fn render_pause_btn(ui: &mut Ui<Event>, state: &State) {
        ui.space(2);

        let label = if state.paused { "resume" } else { "pause" };
        let enabled = state.config.can_pause;

        Button::new(KeyCode::Char(' '), label)
            .throwing(Event::TogglePause)
            .enabled(enabled)
            .render(ui);
    }

    fn render_help_btn(ui: &mut Ui<Event>, state: &State) {
        ui.space(2);

        Button::new(KeyCode::Char('h'), "help")
            .throwing(Event::OpenHelpModal)
            .enabled(state.help.is_some())
            .render(ui);
    }

    fn render_bots_btn(ui: &mut Ui<Event>, state: &State) {
        if !state.config.hero_mode {
            ui.space(2);

            Button::new(KeyCode::Char('b'), "bots")
                .throwing(Event::OpenBotsModal)
                .render(ui);
        }
    }

    fn render_overclock_btn(ui: &mut Ui<Event>, state: &State) {
        if state.config.can_overclock {
            ui.space(2);

            Button::multi("overclock")
                .option(
                    KeyCode::Char('1'),
                    Event::Overclock(ClockSpeed::Normal),
                )
                .option(
                    KeyCode::Char('2'),
                    Event::Overclock(ClockSpeed::Faster),
                )
                .option(
                    KeyCode::Char('3'),
                    Event::Overclock(ClockSpeed::Fastest),
                )
                .render(ui);
        }
    }

    fn render_debug_btn(ui: &mut Ui<Event>, store: &Store) {
        if store.debugging() {
            ui.space(2);

            Button::new(KeyCode::Tab, "debug")
                .throwing(Event::EnableDebugMode)
                .render(ui);
        }
    }

    fn render_status(ui: &mut Ui<Event>, state: &State) {
        let span = if state.paused {
            Some(Span::raw("paused").fg(theme::FG).bg(theme::RED))
        } else if let Some((status, status_tt)) = &state.status {
            let span = Span::raw(status);

            if status_tt.elapsed().as_millis() % 1000 <= 500 {
                Some(span.fg(theme::BG).bg(theme::YELLOW))
            } else {
                Some(span.fg(theme::YELLOW))
            }
        } else {
            let speed = match state.speed {
                ClockSpeed::Normal => None,
                ClockSpeed::Faster => Some("spd:fast"),
                ClockSpeed::Fastest => Some("spd:faster"),
                ClockSpeed::Unlimited => Some("spd:∞"),
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

        ui.clamp(area, |ui| {
            span.render(ui);
        });
    }
}

use super::{Event, State};
use kartoffels_ui::{theme, Button, Render, Ui};
use ratatui::prelude::Rect;
use ratatui::style::Stylize;
use ratatui::text::Span;
use termwiz::input::KeyCode;

#[derive(Debug)]
pub struct BottomPanel;

impl BottomPanel {
    pub fn render(ui: &mut Ui<Event>, state: &State) {
        ui.row(|ui| {
            Self::render_go_back_btn(ui);
            Self::render_pause_btn(ui, state);
            Self::render_help_btn(ui, state);
            Self::render_bots_btn(ui, state);
            Self::render_speed_btn(ui, state);
        });

        Self::render_status(ui, state);
    }

    fn render_go_back_btn(ui: &mut Ui<Event>) {
        Button::new(KeyCode::Escape, "go back")
            .throwing(Event::GoBack)
            .render(ui);
    }

    fn render_pause_btn(ui: &mut Ui<Event>, state: &State) {
        ui.space(2);

        let label = if state.paused { "resume" } else { "pause" };

        let enabled =
            state.handle.is_some() && state.perms.user_can_pause_world;

        Button::new(KeyCode::Char(' '), label)
            .throwing(Event::TogglePause)
            .enabled(enabled)
            .render(ui);
    }

    fn render_help_btn(ui: &mut Ui<Event>, state: &State) {
        ui.space(2);

        Button::new(KeyCode::Char('h'), "help")
            .throwing(Event::ShowHelpDialog)
            .enabled(state.help.is_some())
            .render(ui);
    }

    fn render_bots_btn(ui: &mut Ui<Event>, state: &State) {
        if !state.perms.single_bot_mode {
            ui.space(2);

            Button::new(KeyCode::Char('b'), "bots")
                .throwing(Event::ShowBotsDialog)
                .enabled(state.handle.is_some())
                .render(ui);
        }
    }

    fn render_speed_btn(ui: &mut Ui<Event>, state: &State) {
        if !state.perms.user_can_alter_speed {
            ui.space(2);

            Button::new(KeyCode::Char('S'), "speed")
                .throwing(Event::ShowSpeedDialog)
                .enabled(state.handle.is_some())
                .render(ui);
        }
    }

    fn render_status(ui: &mut Ui<Event>, state: &State) {
        if state.paused {
            let area = Rect {
                x: ui.area().width - 6,
                y: ui.area().y,
                width: 6,
                height: 1,
            };

            ui.clamp(area, |ui| {
                Span::raw("PAUSED").fg(theme::FG).bg(theme::RED).render(ui);
            });
        } else if let Some((status, status_tt)) = &state.status {
            let width = status.len() as u16;

            let area = Rect {
                x: ui.area().width - width,
                y: ui.area().y,
                width,
                height: 1,
            };

            ui.clamp(area, |ui| {
                let span = Span::raw(status);

                let span = if status_tt.elapsed().as_millis() % 1000 <= 500 {
                    span.fg(theme::BG).bg(theme::YELLOW)
                } else {
                    span.fg(theme::YELLOW)
                };

                span.render(ui);
            });
        }
    }
}

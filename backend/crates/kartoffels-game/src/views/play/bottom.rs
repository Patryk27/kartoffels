use super::{Event, State};
use kartoffels_ui::{theme, Button, Ui};
use ratatui::prelude::Rect;
use ratatui::style::Stylize;
use ratatui::text::Span;
use ratatui::widgets::Widget;
use termwiz::input::KeyCode;

#[derive(Debug)]
pub struct BottomPanel;

impl BottomPanel {
    pub fn render(ui: &mut Ui, state: &State) {
        ui.row(|ui| {
            ui.space(2);

            Self::render_go_back_btn(ui);
            Self::render_pause_btn(ui, state);
            Self::render_help_btn(ui, state);
            Self::render_bots_btn(ui, state);
        });

        Self::render_status(ui, state);
    }

    fn render_go_back_btn(ui: &mut Ui) {
        Button::new(KeyCode::Escape, "go back")
            .throwing(Event::GoBack)
            .render(ui);
    }

    fn render_pause_btn(ui: &mut Ui, state: &State) {
        ui.space(2);

        let label = if state.paused { "resume" } else { "pause" };

        let enabled =
            state.handle.is_some() && state.perms.user_can_pause_world;

        Button::new(KeyCode::Char(' '), label)
            .throwing(Event::TogglePause)
            .enabled(enabled)
            .render(ui);
    }

    fn render_help_btn(ui: &mut Ui, state: &State) {
        ui.space(2);

        Button::new(KeyCode::Char('h'), "help")
            .throwing(Event::ShowHelpDialog)
            .enabled(state.help.is_some())
            .render(ui);
    }

    fn render_bots_btn(ui: &mut Ui, state: &State) {
        if !state.perms.single_bot_mode {
            ui.space(2);

            Button::new(KeyCode::Char('b'), "bots")
                .throwing(Event::ShowBotsDialog)
                .enabled(state.handle.is_some())
                .render(ui);
        }
    }

    fn render_status(ui: &mut Ui, state: &State) {
        if state.paused {
            let area = Rect {
                x: ui.area().width - 6,
                y: ui.area().y,
                width: 6,
                height: 1,
            };

            Span::raw("PAUSED")
                .fg(theme::FG)
                .bg(theme::RED)
                .render(area, ui.buf());
        } else if let Some(status) = &state.status {
            let width = status.len() as u16;

            let area = Rect {
                x: ui.area().width - width,
                y: ui.area().y,
                width,
                height: 1,
            };

            Span::raw(status)
                .fg(theme::BG)
                .bg(theme::YELLOW)
                .render(area, ui.buf());
        }
    }
}

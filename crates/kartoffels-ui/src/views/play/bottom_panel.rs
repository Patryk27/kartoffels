use crate::{theme, Button, Ui};
use ratatui::prelude::Rect;
use ratatui::style::Stylize;
use ratatui::text::Span;
use ratatui::widgets::Widget;
use termwiz::input::KeyCode;

#[derive(Debug)]
pub struct BottomPanel;

impl BottomPanel {
    pub fn render(
        ui: &mut Ui,
        paused: bool,
        enabled: bool,
    ) -> Option<BottomPanelEvent> {
        let mut event = None;

        ui.row(|ui| {
            if Button::new(KeyCode::Escape, "quit", enabled)
                .render(ui)
                .activated
            {
                event = Some(BottomPanelEvent::Quit);
            }

            ui.step(1);

            if Button::new(KeyCode::Char('h'), "help", enabled)
                .render(ui)
                .activated
            {
                event = Some(BottomPanelEvent::Help);
            }

            ui.step(1);

            if Button::new(KeyCode::Char('p'), "pause", enabled)
                .render(ui)
                .activated
            {
                event = Some(BottomPanelEvent::Pause);
            }

            ui.step(1);

            if Button::new(KeyCode::Char('b'), "list bots", enabled)
                .render(ui)
                .activated
            {
                event = Some(BottomPanelEvent::ListBots);
            }
        });

        if paused {
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
        }

        event
    }
}

#[derive(Debug)]
pub enum BottomPanelEvent {
    Quit,
    Help,
    Pause,
    ListBots,
}

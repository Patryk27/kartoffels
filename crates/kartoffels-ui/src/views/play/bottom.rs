use crate::{theme, Action};
use ratatui::prelude::{Buffer, Rect};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;
use termwiz::input::{InputEvent, KeyCode, Modifiers};

#[derive(Debug)]
pub struct BottomPanel {
    pub paused: bool,
    pub enabled: bool,
}

impl BottomPanel {
    pub fn handle(event: InputEvent, paused: bool) -> BottomPanelOutcome {
        let InputEvent::Key(event) = event else {
            return BottomPanelOutcome::Forward;
        };

        match (event.key, event.modifiers) {
            (KeyCode::Escape, _) => {
                if paused {
                    BottomPanelOutcome::Pause
                } else {
                    BottomPanelOutcome::Quit
                }
            }

            (KeyCode::Char('p'), Modifiers::NONE) => BottomPanelOutcome::Pause,

            (KeyCode::Char('h' | '?'), Modifiers::NONE) => {
                BottomPanelOutcome::Help
            }

            _ => BottomPanelOutcome::Forward,
        }
    }
}

impl Widget for BottomPanel {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let quit = Action::new("esc", "quit", self.enabled);
        let help = Action::new("h", "help", self.enabled);
        let pause = Action::new("p", "pause", self.enabled);
        let bots = Action::new("b", "bots", self.enabled);
        let sep = [Span::raw("  ")];

        quit.into_iter()
            .chain(sep.clone())
            .chain(help)
            .chain(sep.clone())
            .chain(pause)
            .chain(sep)
            .chain(bots)
            .collect::<Line>()
            .render(area, buf);

        if self.paused {
            let area = Rect {
                x: area.width - 6,
                y: area.y,
                width: 6,
                height: 1,
            };

            Span::raw("PAUSED")
                .fg(theme::FG)
                .bg(theme::RED)
                .render(area, buf);
        }
    }
}

#[derive(Debug)]
pub enum BottomPanelOutcome {
    Quit,
    Pause,
    Help,
    Forward,
}

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
    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        let quit = Action::new("esc", "quit", self.enabled);
        let help = Action::new("h", "help", self.enabled);
        let pause = Action::new("p", "pause", self.enabled);
        let bots = Action::new("b", "list bots", self.enabled);
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

    pub fn handle(
        event: InputEvent,
        paused: bool,
    ) -> Option<BottomPanelOutcome> {
        if let InputEvent::Key(event) = event {
            match (event.key, event.modifiers) {
                (KeyCode::Escape, _) => {
                    return Some(if paused {
                        BottomPanelOutcome::Pause
                    } else {
                        BottomPanelOutcome::Quit
                    })
                }

                (KeyCode::Char('h' | '?'), Modifiers::NONE) => {
                    return Some(BottomPanelOutcome::Help);
                }

                (KeyCode::Char('p'), Modifiers::NONE) => {
                    return Some(BottomPanelOutcome::Pause);
                }

                (KeyCode::Char('b'), Modifiers::NONE) => {
                    return Some(BottomPanelOutcome::ListBots);
                }

                _ => (),
            }
        }

        None
    }
}

#[derive(Debug)]
pub enum BottomPanelOutcome {
    Quit,
    Help,
    Pause,
    ListBots,
}

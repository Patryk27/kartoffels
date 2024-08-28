use crate::Action;
use ratatui::layout::{Constraint, Layout};
use ratatui::prelude::{Buffer, Rect};
use ratatui::widgets::{Paragraph, Widget};
use termwiz::input::{InputEvent, KeyCode, Modifiers};

#[derive(Debug)]
pub struct IdleSidePanel;

impl IdleSidePanel {
    pub fn handle(event: InputEvent) -> IdleSidePanelOutcome {
        if let InputEvent::Key(event) = &event {
            match (event.key, event.modifiers) {
                (KeyCode::Char('c'), Modifiers::NONE) => {
                    return IdleSidePanelOutcome::ConnectToBot;
                }

                (KeyCode::Char('u'), Modifiers::NONE) => {
                    return IdleSidePanelOutcome::UploadBot;
                }

                _ => (),
            }
        }

        IdleSidePanelOutcome::Forward(event)
    }

    pub fn render(self, area: Rect, buf: &mut Buffer, enabled: bool) {
        let [_, upload_area, connect_area] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas(area);

        Paragraph::new(Action::new("u", "upload bot", enabled))
            .render(upload_area, buf);

        Paragraph::new(Action::new("c", "connect to bot", enabled))
            .render(connect_area, buf);
    }
}

#[derive(Debug)]
pub enum IdleSidePanelOutcome {
    ConnectToBot,
    UploadBot,
    Forward(InputEvent),
}

use super::SidePanelEvent;
use crate::Action;
use ratatui::layout::{Constraint, Layout};
use ratatui::prelude::{Buffer, Rect};
use ratatui::widgets::{Paragraph, Widget};
use termwiz::input::{InputEvent, KeyCode, Modifiers};

#[derive(Debug, Default)]
pub struct IdleSidePanel {
    pub enabled: bool,
}

impl IdleSidePanel {
    pub fn render(self, area: Rect, buf: &mut Buffer) {
        let [_, upload_area, connect_area] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas(area);

        Paragraph::new(Action::new("u", "upload bot", self.enabled))
            .render(upload_area, buf);

        Paragraph::new(Action::new("c", "connect to bot", self.enabled))
            .render(connect_area, buf);
    }

    pub fn handle(event: InputEvent) -> SidePanelEvent {
        if let InputEvent::Key(event) = &event {
            match (event.key, event.modifiers) {
                (KeyCode::Char('c'), Modifiers::NONE) => {
                    return SidePanelEvent::ConnectToBot;
                }

                (KeyCode::Char('u'), Modifiers::NONE) => {
                    return SidePanelEvent::UploadBot;
                }

                _ => (),
            }
        }

        SidePanelEvent::Forward(event)
    }
}

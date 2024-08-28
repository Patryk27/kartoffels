use kartoffels_world::prelude::BotId;
use ratatui::buffer::Buffer;
use ratatui::layout::{Offset, Rect};
use ratatui::widgets::{Paragraph, Widget};
use termwiz::input::{InputEvent, KeyCode, Modifiers};

#[derive(Debug, Default)]
pub struct ConnectingSidePanel {
    pub id: String,
}

impl ConnectingSidePanel {
    pub fn handle(&mut self, event: InputEvent) -> ConnectingSidePanelOutcome {
        if let InputEvent::Key(event) = &event {
            match (event.key, event.modifiers) {
                (KeyCode::Char(ch), Modifiers::NONE) => {
                    if ch.is_alphanumeric() || ch == '-' {
                        // TODO limit length
                        self.id.push(ch);
                    }

                    return ConnectingSidePanelOutcome::None;
                }

                (KeyCode::Backspace, Modifiers::NONE) => {
                    self.id.pop();

                    return ConnectingSidePanelOutcome::None;
                }

                (KeyCode::Enter, Modifiers::NONE) => {
                    if let Ok(id) = self.id.parse() {
                        return ConnectingSidePanelOutcome::ConnectToBot(id);
                    } else {
                        // TODO warn
                        return ConnectingSidePanelOutcome::None;
                    }
                }

                (KeyCode::Escape, Modifiers::NONE) => {
                    return ConnectingSidePanelOutcome::Abort;
                }

                _ => (),
            }
        }

        ConnectingSidePanelOutcome::Forward(event)
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("enter bot id:").render(area, buf);

        Paragraph::new(format!("> {}", &self.id))
            .render(area.offset(Offset { x: 0, y: 1 }), buf);
    }
}

#[derive(Debug)]
pub enum ConnectingSidePanelOutcome {
    ConnectToBot(BotId),
    Abort,
    None,
    Forward(InputEvent),
}

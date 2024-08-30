use super::DialogEvent;
use crate::{Action, BlockExt, LayoutExt, RectExt};
use ratatui::layout::{ Layout};
use ratatui::prelude::{Buffer, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph, Widget};
use termwiz::input::{InputEvent, KeyCode};

#[derive(Debug)]
pub struct ErrorDialog {
    pub error: String,
}

impl ErrorDialog {
    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        let text = Paragraph::new(self.error.as_str()).wrap(Default::default());

        let width = 50;
        let height = text.line_count(width) as u16;

        let area = Block::dialog_error(
            Some(" whoopsie "),
            Layout::dialog(width, height + 2, area),
            buf,
        );

        text.render(area, buf);

        Line::from(Action::new("enter", "close", true))
            .right_aligned()
            .render(area.footer(), buf);
    }

    pub fn handle(&mut self, event: InputEvent) -> Option<DialogEvent> {
        if let InputEvent::Key(event) = event {
            if event.key == KeyCode::Enter {
                return Some(DialogEvent::Close);
            }
        }

        None
    }
}

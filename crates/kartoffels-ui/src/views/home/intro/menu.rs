use crate::Action;
use ratatui::prelude::{Buffer, Rect};
use ratatui::text::Text;
use ratatui::widgets::Widget;

#[derive(Debug)]
pub struct Menu;

impl Menu {
    pub const HEIGHT: u16 = 5;
}

impl Widget for Menu {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut text = Text::default();

        text.push_line(Action::new("p", "play", true));
        text.push_line(Action::new("t", "see tutorial", true));
        text.push_line(Action::new("c", "see challenges", true));
        text.push_line("");
        text.push_line(Action::new("esc", "quit", true));

        text.centered().render(area, buf);
    }
}

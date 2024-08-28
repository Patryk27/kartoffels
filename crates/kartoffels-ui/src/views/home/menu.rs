use crate::{theme, Action};
use kartoffels_world::prelude::Handle as WorldHandle;
use ratatui::prelude::{Buffer, Rect};
use ratatui::style::Stylize;
use ratatui::text::Text;
use ratatui::widgets::Widget;

#[derive(Debug)]
pub struct Menu<'a> {
    pub blink: bool,
    pub worlds: &'a [&'a WorldHandle],
}

impl<'a> Menu<'a> {
    pub const WIDTH: u16 = 32;

    pub fn height(&self) -> u16 {
        (6 + self.worlds.len()) as u16
    }
}

impl Widget for Menu<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut text = Text::default();

        text.push_line(Action::new("t", "see tutorial", true));
        text.push_line(Action::new("c", "see challenges", true));
        text.push_line(Action::new("esc", "quit", true));
        text.push_line("");

        for (idx, world) in self.worlds.iter().enumerate() {
            text.push_line(Action::new(
                (idx + 1).to_string(),
                format!("play: {}", world.name()),
                true,
            ));
        }

        text.push_line("");
        text.push_line("$ ");
        text.push_span(if self.blink { " " } else { "_" }.fg(theme::GREEN));

        text.centered().render(area, buf);
    }
}

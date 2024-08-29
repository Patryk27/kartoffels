use crate::{Action, Prompt};
use ratatui::prelude::{Buffer, Rect};
use ratatui::text::Text;
use ratatui::widgets::Widget;

#[derive(Debug)]
pub struct Menu {
    prompt: Prompt,
}

impl Menu {
    pub const HEIGHT: u16 = 7;

    pub fn new() -> Self {
        Self {
            prompt: Prompt::new(),
        }
    }

    pub async fn tick(&mut self) {
        self.prompt.tick().await;
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        let mut text = Text::default();

        text.push_line(Action::new("p", "play", true));
        text.push_line(Action::new("t", "see tutorial", true));
        text.push_line(Action::new("c", "see challenges", true));
        text.push_line("");
        text.push_line(Action::new("esc", "quit", true));
        text.push_line("");
        text.push_line(self.prompt.as_line());

        text.centered().render(area, buf);
    }
}

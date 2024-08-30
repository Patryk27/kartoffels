use crate::{theme, Ui};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;
use tokio::time::{self, Interval};

#[derive(Debug)]
pub struct Prompt {
    visible: bool,
    interval: Interval,
}

impl Prompt {
    pub fn render(&mut self, ui: &mut Ui) {
        if ui.poll(self.interval.tick()).is_ready() {
            self.visible = !self.visible;

            _ = ui.poll(self.interval.tick());
        }

        self.as_line().centered().render(ui.area(), ui.buf());
    }

    fn as_line(&self) -> Line<'static> {
        Line::from_iter([
            Span::raw("$ "),
            if self.visible { "_" } else { " " }.fg(theme::GREEN),
        ])
    }
}

impl Default for Prompt {
    fn default() -> Self {
        Self {
            visible: true,
            interval: time::interval(theme::CARET_INTERVAL),
        }
    }
}

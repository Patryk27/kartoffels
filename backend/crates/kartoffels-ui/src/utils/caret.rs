use crate::{theme, Ui};
use ratatui::style::Stylize;
use ratatui::text::Span;
use std::time::Duration;
use tokio::time::{self, Interval};

#[derive(Debug)]
pub struct Caret {
    visible: bool,
    interval: Interval,
}

impl Caret {
    pub fn as_span(&mut self, ui: &mut Ui) -> Span {
        if ui.poll_interval(&mut self.interval) {
            self.visible = !self.visible;
        }

        Span::raw(if self.visible { "_" } else { "" }).fg(theme::GREEN)
    }
}

impl Default for Caret {
    fn default() -> Self {
        Self {
            visible: false,
            interval: time::interval(Duration::from_millis(500)),
        }
    }
}

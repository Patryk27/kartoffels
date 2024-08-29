use super::IntervalExt;
use crate::theme;
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use tokio::time::{self, Interval};

#[derive(Debug)]
pub struct Prompt {
    visible: bool,
    interval: Interval,
}

impl Prompt {
    pub async fn tick(&mut self) {
        self.interval.tick().await;
        self.visible = !self.visible;
    }

    pub fn as_line(&self) -> Line<'static> {
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
            interval: time::interval(theme::CARET_INTERVAL).skipping_first(),
        }
    }
}

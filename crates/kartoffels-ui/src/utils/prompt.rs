use crate::theme;
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use std::time::Duration;
use tokio::time::{self, Interval};

#[derive(Debug)]
pub struct Prompt {
    visible: bool,
    interval: Interval,
}

impl Prompt {
    pub fn new() -> Self {
        let mut interval = time::interval(Duration::from_millis(500));

        _ = interval.tick();

        Self {
            visible: false,
            interval,
        }
    }

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

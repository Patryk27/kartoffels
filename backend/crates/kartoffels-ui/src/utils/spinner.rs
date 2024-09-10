use crate::{theme, Ui};
use ratatui::style::Stylize;
use ratatui::text::Span;
use std::time::Duration;
use tokio::time::{self, Interval};

#[derive(Debug)]
pub struct Spinner {
    icon: usize,
    interval: Interval,
}

impl Spinner {
    const ICONS: &[&str] = &["|", "/", "-", "\\"];

    pub fn as_span(&mut self, ui: &mut Ui) -> Span {
        if ui.poll_interval(&mut self.interval) {
            self.icon += 1;
        }

        Span::raw(Self::ICONS[self.icon % Self::ICONS.len()]).fg(theme::GREEN)
    }
}

impl Default for Spinner {
    fn default() -> Self {
        Self {
            icon: 0,
            interval: time::interval(Duration::from_millis(250)),
        }
    }
}

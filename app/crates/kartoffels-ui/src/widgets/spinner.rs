use crate::theme;
use ratatui::style::Stylize;
use ratatui::text::Span;
use std::time::Instant;

#[derive(Debug)]
pub struct Spinner {
    instant: Instant,
}

impl Spinner {
    const ICONS: &[&str] = &["|", "/", "-", "\\"];

    pub fn as_span(&mut self) -> Span {
        let icon = self.instant.elapsed().as_millis() / 250;
        let icon = (icon as usize) % Self::ICONS.len();

        Span::raw(Self::ICONS[icon]).fg(theme::GREEN)
    }
}

impl Default for Spinner {
    fn default() -> Self {
        Self {
            instant: Instant::now(),
        }
    }
}

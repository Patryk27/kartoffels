use crate::theme;
use ratatui::style::Stylize;
use ratatui::text::Span;
use std::time::Instant;

#[derive(Debug)]
pub struct Caret {
    instant: Instant,
}

impl Caret {
    pub fn as_span(&mut self) -> Span {
        let visible = self.instant.elapsed().as_millis() % 1000 <= 500;

        Span::raw(if visible { "_" } else { "" }).fg(theme::GREEN)
    }
}

impl Default for Caret {
    fn default() -> Self {
        Self {
            instant: Instant::now(),
        }
    }
}

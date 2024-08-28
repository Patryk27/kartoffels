use super::theme;
use ratatui::prelude::{Buffer, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::Widget;
use std::borrow::Cow;

#[derive(Debug)]
pub struct Clear;

impl Widget for Clear {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for x in area.left()..area.right() {
            for y in area.top()..area.bottom() {
                buf[(x, y)]
                    .set_symbol(" ")
                    .set_fg(theme::FG)
                    .set_bg(theme::BG);
            }
        }
    }
}

#[derive(Debug)]
pub struct Action<'a> {
    pub key: Cow<'a, str>,
    pub desc: Cow<'a, str>,
    pub enabled: bool,
}

impl<'a> Action<'a> {
    pub fn new(
        key: impl Into<Cow<'a, str>>,
        desc: impl Into<Cow<'a, str>>,
        enabled: bool,
    ) -> Self {
        Self {
            key: key.into(),
            desc: desc.into(),
            enabled,
        }
    }
}

impl<'a> IntoIterator for Action<'a> {
    type Item = Span<'a>;
    type IntoIter = impl Iterator<Item = Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let s1 = if self.enabled {
            Style::default()
        } else {
            Style::new().fg(theme::DARK_GRAY)
        };

        let s2 = if self.enabled {
            Style::new().bold().fg(theme::GREEN)
        } else {
            Style::new().fg(theme::DARK_GRAY)
        };

        [
            Span::styled("[", s1),
            Span::styled(self.key, s2),
            Span::styled("] ", s1),
            Span::styled(self.desc, s1),
        ]
        .into_iter()
    }
}

impl<'a> From<Action<'a>> for Line<'a> {
    fn from(this: Action<'a>) -> Self {
        this.into_iter().collect()
    }
}

impl<'a> From<Action<'a>> for Text<'a> {
    fn from(this: Action<'a>) -> Self {
        Text::from(Line::from(this))
    }
}

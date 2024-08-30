use crate::{theme, Ui};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;
use std::borrow::Cow;
use termwiz::input::{KeyCode, Modifiers};

#[derive(Debug)]
pub struct Button<'a> {
    pub key: KeyCode,
    pub desc: Cow<'a, str>,
    pub enabled: bool,
}

impl<'a> Button<'a> {
    pub fn new(
        key: KeyCode,
        desc: impl Into<Cow<'a, str>>,
        enabled: bool,
    ) -> Self {
        Self {
            key,
            desc: desc.into(),
            enabled,
        }
    }

    pub fn width(&self) -> u16 {
        (Self::key_name(self.key).len() + self.desc.len() + 3) as u16
    }

    pub fn render(self, ui: &mut Ui) -> ButtonResponse {
        let response = ButtonResponse {
            activated: self.enabled && ui.key(self.key, Modifiers::NONE),
        };

        Line::from_iter(self).centered().render(ui.area(), ui.buf());

        response
    }

    fn key_name(key: KeyCode) -> String {
        match key {
            KeyCode::Char(ch) => ch.to_string(),
            KeyCode::Enter => "enter".into(),
            KeyCode::Escape => "esc".into(),

            key => unimplemented!("key={:?}", key),
        }
    }
}

impl<'a> IntoIterator for Button<'a> {
    type Item = Span<'a>;
    type IntoIter = impl Iterator<Item = Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let key = Self::key_name(self.key);
        let desc = self.desc;

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
            Span::styled(key, s2),
            Span::styled("] ", s1),
            Span::styled(desc, s1),
        ]
        .into_iter()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ButtonResponse {
    pub activated: bool,
}

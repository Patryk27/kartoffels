use crate::{theme, Ui};
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;
use std::borrow::Cow;
use termwiz::input::{KeyCode, Modifiers};

#[derive(Debug)]
pub struct Button<'a> {
    pub key: KeyCode,
    pub desc: Cow<'a, str>,
    pub alignment: Alignment,
    pub enabled: bool,
    pub relative: bool,
}

impl<'a> Button<'a> {
    pub fn new(key: KeyCode, desc: impl Into<Cow<'a, str>>) -> Self {
        Self {
            key,
            desc: desc.into(),
            alignment: Alignment::Left,
            enabled: true,
            relative: false,
        }
    }

    pub fn centered(mut self) -> Self {
        self.alignment = Alignment::Center;
        self
    }

    pub fn right(mut self) -> Self {
        self.alignment = Alignment::Right;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn relative(mut self) -> Self {
        self.relative = true;
        self
    }

    pub fn width(&self) -> u16 {
        (Self::key_name(self.key).len() + self.desc.len() + 3) as u16
    }

    pub fn render(self, ui: &mut Ui) -> ButtonResponse {
        let area = self.layout(ui);
        let response = self.response(ui, area);
        let (style_key, style_desc) = self.style(&response);

        let key = Button::key_name(self.key);
        let desc = &*self.desc;

        Line::from_iter([
            Span::styled("[", style_desc),
            Span::styled(key, style_key),
            Span::styled("] ", style_desc),
            Span::styled(desc, style_desc),
        ])
        .render(area, ui.buf());

        if self.relative {
            ui.step(area.width);
        }

        response
    }

    fn layout(&self, ui: &Ui) -> Rect {
        let area = ui.area();
        let width = self.width();

        let x = match self.alignment {
            Alignment::Left => area.x,
            Alignment::Center => area.x + (area.width - width) / 2,
            Alignment::Right => area.x + area.width - width,
        };

        Rect {
            x,
            y: area.y,
            width,
            height: 1,
        }
    }

    fn response(&self, ui: &Ui, area: Rect) -> ButtonResponse {
        let hovered = self.enabled && ui.mouse_over(area);
        let pressed_mouse = hovered && ui.mouse_pressed();
        let pressed_key = self.enabled && ui.key(self.key, Modifiers::NONE);

        ButtonResponse {
            hovered,
            pressed: pressed_mouse || pressed_key,
        }
    }

    fn style(&self, response: &ButtonResponse) -> (Style, Style) {
        let key = if self.enabled {
            if response.pressed || response.hovered {
                Style::new().bold().bg(theme::GREEN).fg(theme::BG)
            } else {
                Style::new().bold().fg(theme::GREEN)
            }
        } else {
            Style::new().fg(theme::DARK_GRAY)
        };

        let desc = if self.enabled {
            if response.pressed || response.hovered {
                Style::new().bg(theme::GREEN).fg(theme::BG)
            } else {
                Style::default()
            }
        } else {
            Style::new().fg(theme::DARK_GRAY)
        };

        (key, desc)
    }

    fn key_name(key: KeyCode) -> String {
        match key {
            KeyCode::Char(ch) => ch.to_string(),
            KeyCode::Enter => "enter".into(),
            KeyCode::Escape => "esc".into(),

            key => unimplemented!("{:?}", key),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ButtonResponse {
    pub hovered: bool,
    pub pressed: bool,
}

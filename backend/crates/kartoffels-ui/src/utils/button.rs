use crate::{theme, Ui, UiWidget};
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Style, Styled, Stylize};
use ratatui::text::{Span, Text};
use std::borrow::Cow;
use termwiz::input::{KeyCode, Modifiers};

#[derive(Clone, Debug)]
pub struct Button<'a, T> {
    label: Cow<'a, str>,
    options: Vec<(Option<KeyCode>, Option<T>)>,
    help: Option<Cow<'a, str>>,
    alignment: Alignment,
    enabled: bool,
    style: Style,
}

impl<'a, T> Button<'a, T> {
    pub fn new(
        key: impl Into<Option<KeyCode>>,
        label: impl Into<Cow<'a, str>>,
    ) -> Self {
        Self {
            label: label.into(),
            options: vec![(key.into(), None)],
            help: None,
            alignment: Alignment::Left,
            enabled: true,
            style: Default::default(),
        }
    }

    pub fn multi(label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            label: label.into(),
            options: Default::default(),
            help: None,
            alignment: Alignment::Left,
            enabled: true,
            style: Default::default(),
        }
    }

    pub fn option(mut self, key: KeyCode, event: T) -> Self {
        self.options.push((Some(key), Some(event)));
        self
    }

    pub fn throwing(mut self, event: T) -> Self {
        self.options.last_mut().unwrap().1 = Some(event);
        self
    }

    pub fn centered(mut self) -> Self {
        self.alignment = Alignment::Center;
        self
    }

    pub fn right_aligned(mut self) -> Self {
        self.alignment = Alignment::Right;
        self
    }

    pub fn help(mut self, help: impl Into<Cow<'a, str>>) -> Self {
        self.help = Some(help.into());
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn width(&self) -> u16 {
        if self.is_mouse_only() {
            self.label.len() as u16 + 2
        } else {
            let keys = self
                .options
                .iter()
                .map(|(key, _)| Self::key_name(key.unwrap()).len() as u16)
                .sum::<u16>();

            let slashes = (self.options.len() - 1) as u16;

            keys + slashes + self.label.len() as u16 + 3
        }
    }

    fn is_mouse_only(&self) -> bool {
        self.options.len() == 1 && self.options[0].0.is_none()
    }

    fn layout(&self, ui: &Ui<T>) -> Rect {
        let width = self.width();

        let x = match self.alignment {
            Alignment::Left => ui.area.x,
            Alignment::Center => ui.area.x + (ui.area.width - width) / 2,
            Alignment::Right => ui.area.x + ui.area.width - width,
        };

        Rect {
            x,
            y: ui.area.y,
            width,
            height: 1,
        }
    }

    fn response(&self, ui: &Ui<T>, area: Rect) -> ButtonResponse {
        if !ui.enabled || !self.enabled {
            return ButtonResponse {
                hovered: false,
                pressed: false,
                option: None,
            };
        }

        let hovered = ui.mouse_over(area);

        let mouse_option = if ui.mouse_over(area) && ui.mouse_pressed() {
            // TODO support multi-buttons
            Some(0)
        } else {
            None
        };

        let kbd_option = self
            .options
            .iter()
            .enumerate()
            .filter_map(|(idx, &(key, _))| {
                let key = key?;

                if ui.key(key, Modifiers::NONE) {
                    Some(idx)
                } else {
                    None
                }
            })
            .next();

        ButtonResponse {
            hovered,
            pressed: mouse_option.is_some() || kbd_option.is_some(),
            option: mouse_option.or(kbd_option),
        }
    }

    fn style(&self, ui: &Ui<T>, response: &ButtonResponse) -> (Style, Style) {
        let key = if ui.enabled && self.enabled {
            if response.pressed || response.hovered {
                Style::new().bold().bg(theme::GREEN).fg(theme::BG)
            } else {
                Style::new().fg(theme::GREEN)
            }
        } else {
            Style::new().fg(theme::DARK_GRAY)
        };

        let label = if ui.enabled && self.enabled {
            if response.pressed || response.hovered {
                Style::new().bg(theme::GREEN).fg(theme::BG)
            } else {
                Style::default()
            }
        } else {
            Style::new().fg(theme::DARK_GRAY)
        };

        (key, label)
    }

    fn key_name(key: KeyCode) -> String {
        match key {
            KeyCode::Char(' ') => "spc".into(),
            KeyCode::Char(ch) => ch.to_string(),
            KeyCode::Tab => "tab".into(),
            KeyCode::Enter => "enter".into(),
            KeyCode::Escape => "esc".into(),

            key => unimplemented!("{:?}", key),
        }
    }
}

impl<T> Styled for Button<'_, T> {
    type Item = Self;

    fn style(&self) -> Style {
        self.style
    }

    fn set_style<S>(mut self, style: S) -> Self::Item
    where
        S: Into<Style>,
    {
        self.style = style.into();
        self
    }
}

impl<T> UiWidget<T> for Button<'_, T> {
    type Response = ButtonResponse;

    fn render(mut self, ui: &mut Ui<T>) -> Self::Response {
        let area = self.layout(ui);
        let resp = self.response(ui, area);
        let (key_style, label_style) = self.style(ui, &resp);
        let is_mouse_only = self.is_mouse_only();

        ui.clamp(area, |ui| {
            ui.row(|ui| {
                if is_mouse_only {
                    ui.span(Span::styled("[", label_style));
                    ui.span(Span::styled(self.label, key_style));
                    ui.span(Span::styled("]", label_style));
                } else {
                    ui.span(Span::styled("[", label_style));

                    for (idx, (key, _)) in self.options.iter().enumerate() {
                        if idx > 0 {
                            ui.span(Span::styled("/", label_style));
                        }

                        ui.span(Span::styled(
                            Self::key_name(key.unwrap()),
                            key_style,
                        ));
                    }

                    ui.span(Span::styled("] ", label_style));

                    ui.span(Span::styled(
                        self.label,
                        label_style.patch(self.style),
                    ));
                }
            });
        });

        if ui.layout.is_row() {
            ui.space(area.width);
        } else {
            ui.space(area.height);
        }

        if let Some(help) = self.help {
            assert!(ui.layout.is_col());

            ui.row(|ui| {
                ui.space(4);
                ui.line(Text::raw(help).fg(theme::GRAY));
            });

            ui.space(1);
        }

        if resp.pressed
            && let Some(idx) = resp.option
            && let Some(event) = self.options.swap_remove(idx).1
        {
            ui.throw(event);
        }

        resp
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ButtonResponse {
    pub hovered: bool,
    pub pressed: bool,
    pub option: Option<usize>,
}

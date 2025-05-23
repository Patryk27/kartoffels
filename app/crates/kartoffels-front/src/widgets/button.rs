use crate::{theme, Ui, UiWidget};
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Style, Styled, Stylize};
use ratatui::text::{Span, Text};
use std::borrow::Cow;
use termwiz::input::{KeyCode, Modifiers};

#[derive(Clone, Debug)]
pub struct Button<'a, T> {
    label: Cow<'a, str>,
    options: Vec<ButtonOption<T>>,
    help: Option<Cow<'a, str>>,
    align: Option<Alignment>,
    style: Style,
}

impl<'a, T> Button<'a, T> {
    pub fn new(
        label: impl Into<Cow<'a, str>>,
        key: impl Into<Option<KeyCode>>,
    ) -> Self {
        Self {
            label: label.into(),
            options: vec![ButtonOption {
                key: key.into(),
                event: None,
            }],
            help: None,
            align: None,
            style: Default::default(),
        }
    }

    pub fn multi(label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            label: label.into(),
            options: Default::default(),
            help: None,
            align: None,
            style: Default::default(),
        }
    }

    pub fn throwing(mut self, event: T) -> Self {
        assert!(self.options.len() == 1);

        self.options[0].event = Some(event);
        self
    }

    pub fn throwing_on(mut self, key: KeyCode, event: T) -> Self {
        self.options.push(ButtonOption {
            key: Some(key),
            event: Some(event),
        });

        self
    }

    pub fn right_aligned(mut self) -> Self {
        self.align = Some(Alignment::Right);
        self
    }

    pub fn help(mut self, help: impl Into<Cow<'a, str>>) -> Self {
        self.help = Some(help.into());
        self
    }

    pub fn width(&self) -> u16 {
        let keys = self
            .options
            .iter()
            .map(|opt| key_name_len(opt.key.unwrap()))
            .sum::<u16>();

        let slashes = (self.options.len() - 1) as u16;

        keys + slashes + self.label.len() as u16 + 3
    }

    fn supports_mouse(&self) -> bool {
        self.options.len() == 1
    }

    fn layout(&self, ui: &Ui<T>) -> Rect {
        let width = self.width();

        let x = match self.align.unwrap_or_else(|| ui.dir.into()) {
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
        if !ui.enabled {
            return Default::default();
        }

        let hovered = self.supports_mouse() && ui.mouse_over(area);

        let mouse_option = if self.supports_mouse()
            && ui.mouse_over(area)
            && ui.mouse_pressed()
        {
            Some(0)
        } else {
            None
        };

        let kbd_option = if ui.focused {
            self.options
                .iter()
                .enumerate()
                .filter_map(|(idx, opt)| {
                    let key = opt.key?;

                    if ui.key(key, Modifiers::NONE) {
                        Some(idx)
                    } else {
                        None
                    }
                })
                .next()
        } else {
            None
        };

        ButtonResponse {
            hovered,
            pressed: mouse_option.is_some() || kbd_option.is_some(),
            option: mouse_option.or(kbd_option),
        }
    }

    fn style(&self, ui: &Ui<T>, response: &ButtonResponse) -> (Style, Style) {
        let key = if ui.enabled && (ui.focused || response.hovered) {
            if response.pressed || response.hovered {
                Style::new().bold().bg(theme::GREEN).fg(theme::BG)
            } else {
                Style::new().fg(theme::GREEN)
            }
        } else {
            Style::new().fg(theme::DARK_GRAY)
        };

        let label = if ui.enabled {
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
        (&mut self).render(ui)
    }
}

impl<T> UiWidget<T> for &mut Button<'_, T> {
    type Response = ButtonResponse;

    fn render(self, ui: &mut Ui<T>) -> Self::Response {
        let area = self.layout(ui);
        let resp = self.response(ui, area);
        let (key_style, label_style) = self.style(ui, &resp);

        ui.at(area, |ui| {
            ui.ltr(|ui| {
                ui.row(|ui| {
                    ui.span(Span::styled("[", label_style));

                    for (idx, opt) in self.options.iter().enumerate() {
                        if idx > 0 {
                            ui.span(Span::styled("/", label_style));
                        }

                        ui.span(Span::styled(
                            key_name(opt.key.unwrap()),
                            key_style,
                        ));
                    }

                    ui.span(Span::styled("] ", label_style));

                    ui.span(Span::styled(
                        self.label.as_str(),
                        label_style.patch(self.style),
                    ));
                });
            });
        });

        if ui.layout.is_row() {
            ui.space(area.width);
        } else {
            ui.space(area.height);
        }

        if let Some(help) = &self.help {
            assert!(ui.layout.is_col());

            ui.row(|ui| {
                ui.space(4);
                ui.line(Text::raw(help.as_str()).fg(theme::GRAY));
            });
        }

        if resp.pressed
            && let Some(idx) = resp.option
            && let Some(event) = self.options[idx].event.take()
        {
            ui.throw(event);
        }

        resp
    }
}

#[derive(Clone, Debug)]
struct ButtonOption<T> {
    key: Option<KeyCode>,
    event: Option<T>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ButtonResponse {
    pub hovered: bool,
    pub pressed: bool,
    pub option: Option<usize>,
}

fn key_name(key: KeyCode) -> Cow<'static, str> {
    match key {
        KeyCode::Char(' ') => Cow::Borrowed("spc"),
        KeyCode::Char(ch) => Cow::Owned(ch.to_string()),
        KeyCode::Tab => Cow::Borrowed("tab"),
        KeyCode::Enter => Cow::Borrowed("enter"),
        KeyCode::Escape => Cow::Borrowed("esc"),
        KeyCode::UpArrow => Cow::Borrowed("↑"),
        KeyCode::DownArrow => Cow::Borrowed("↓"),
        key => unimplemented!("{:?}", key),
    }
}

fn key_name_len(key: KeyCode) -> u16 {
    match key {
        KeyCode::Char(' ') => 3,
        KeyCode::Char(_) => 1,
        KeyCode::Tab => 3,
        KeyCode::Enter => 5,
        KeyCode::Escape => 3,
        KeyCode::UpArrow => 1,
        KeyCode::DownArrow => 1,
        key => unimplemented!("{:?}", key),
    }
}

use crate::{Button, Ui};
use ratatui::style::{Style, Styled};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Paragraph, WidgetRef};
use std::cmp;
use termwiz::input::KeyCode;

#[derive(Clone, Debug)]
pub struct Dialog<'a, T> {
    pub title: Option<&'a str>,
    pub body: Vec<DialogLine<'a>>,
    pub buttons: Vec<DialogButton<'a, T>>,
}

impl<'a, T> Dialog<'a, T>
where
    T: Clone,
{
    pub fn render(&self, ui: &mut Ui) -> Option<T> {
        let mut resp = None;

        let body = {
            let text: Text = self
                .body
                .iter()
                .filter(|line| line.matches(ui))
                .map(|line| line.inner.clone())
                .collect();

            Paragraph::new(text).wrap(Default::default())
        };

        let width = cmp::min(60, ui.area().width - 4);
        let height = body.line_count(width) as u16 + 2;

        ui.info_window(width, height, self.title, |ui| {
            body.render_ref(ui.area(), ui.buf());
            ui.space(height - 1);

            for button in &self.buttons {
                if button.btn.render(ui).pressed {
                    resp = Some(button.resp.clone());
                }
            }
        });

        resp
    }
}

#[derive(Clone, Debug)]
pub struct DialogLine<'a> {
    inner: Line<'a>,
    cond: Option<DialogLineCondition>,
}

impl<'a> DialogLine<'a> {
    pub fn raw(content: &'a str) -> Self {
        Self {
            inner: Line::raw(content),
            cond: None,
        }
    }

    pub fn ssh(content: &'a str) -> Self {
        Self {
            inner: Line::raw(content),
            cond: Some(DialogLineCondition::ShowOnlyOnSsh),
        }
    }

    pub fn web(content: &'a str) -> Self {
        Self {
            inner: Line::raw(content),
            cond: Some(DialogLineCondition::ShowOnlyOnWeb),
        }
    }

    pub fn right_aligned(mut self) -> Self {
        self.inner = self.inner.right_aligned();
        self
    }

    fn matches(&self, ui: &Ui) -> bool {
        match self.cond {
            Some(DialogLineCondition::ShowOnlyOnSsh) => ui.ty().is_ssh(),
            Some(DialogLineCondition::ShowOnlyOnWeb) => ui.ty().is_web(),
            None => true,
        }
    }
}

impl<'a> Styled for DialogLine<'a> {
    type Item = Self;

    fn style(&self) -> Style {
        Styled::style(&self.inner)
    }

    fn set_style<S>(self, style: S) -> Self::Item
    where
        S: Into<Style>,
    {
        Self {
            inner: self.inner.set_style(style),
            cond: self.cond,
        }
    }
}

impl<'a> FromIterator<Span<'a>> for DialogLine<'a> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Span<'a>>,
    {
        Self {
            inner: iter.into_iter().collect(),
            cond: None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum DialogLineCondition {
    ShowOnlyOnSsh,
    ShowOnlyOnWeb,
}

#[derive(Clone, Debug)]
pub struct DialogButton<'a, T> {
    btn: Button<'a>,
    resp: T,
}

impl<'a, T> DialogButton<'a, T> {
    pub fn new(key: KeyCode, label: &'a str, resp: T) -> Self {
        Self {
            btn: Button::new(key, label),
            resp,
        }
    }

    pub fn abort(label: &'a str, resp: T) -> Self {
        Self::new(KeyCode::Escape, label, resp)
    }

    pub fn confirm(label: &'a str, resp: T) -> Self {
        Self::new(KeyCode::Enter, label, resp).right_aligned()
    }

    pub fn right_aligned(mut self) -> Self {
        self.btn = self.btn.right_aligned();
        self
    }
}

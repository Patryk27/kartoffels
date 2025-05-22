use crate::{Button, FromMarkdown, Ui, UiWidget};
use ratatui::layout::Alignment;
use ratatui::style::{Style, Styled};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::Paragraph;
use std::cmp;
use termwiz::input::KeyCode;

#[derive(Clone, Debug)]
pub struct Msg<T = ()> {
    pub title: Option<&'static str>,
    pub body: Vec<MsgLine>,
    pub buttons: Vec<MsgButton<T>>,
}

impl<T> UiWidget<T> for &Msg<T>
where
    T: Clone,
{
    type Response = ();

    fn render(self, ui: &mut Ui<T>) -> Self::Response {
        let body = {
            let text: Text = self
                .body
                .iter()
                .filter(|line| line.matches(ui))
                .map(|line| line.inner.clone())
                .collect();

            Paragraph::new(text).wrap(Default::default())
        };

        let width = cmp::min(60, ui.area.width - 4);
        let height = body.line_count(width) as u16 + 2;

        ui.imodal(width, height, self.title, |ui| {
            ui.add(&body);
            ui.space(height - 1);

            ui.row(|ui| {
                let left_btns = self
                    .buttons
                    .iter()
                    .filter(|btn| btn.align == Alignment::Left);

                let right_btns = self
                    .buttons
                    .iter()
                    .filter(|btn| btn.align == Alignment::Right)
                    .rev();

                for btn in left_btns {
                    ui.add(btn.inner.clone());
                    ui.space(1);
                }

                ui.rtl(|ui| {
                    for button in right_btns {
                        ui.add(button.inner.clone());
                        ui.space(1);
                    }
                });
            });
        });
    }
}

#[derive(Clone, Debug)]
pub struct MsgLine {
    inner: Line<'static>,
    cond: Option<MsgLineCondition>,
}

impl MsgLine {
    pub fn new(content: impl AsRef<str>) -> Self {
        Self {
            inner: Line::md(content.as_ref()),
            cond: None,
        }
    }

    pub fn ssh(content: impl AsRef<str>) -> Self {
        Self {
            cond: Some(MsgLineCondition::ShowOnlyOnSsh),
            ..Self::new(content)
        }
    }

    pub fn web(content: impl AsRef<str>) -> Self {
        Self {
            cond: Some(MsgLineCondition::ShowOnlyOnWeb),
            ..Self::new(content)
        }
    }

    pub fn centered(mut self) -> Self {
        self.inner = self.inner.centered();
        self
    }

    pub fn right_aligned(mut self) -> Self {
        self.inner = self.inner.right_aligned();
        self
    }

    fn matches<E>(&self, ui: &Ui<E>) -> bool {
        match self.cond {
            Some(MsgLineCondition::ShowOnlyOnSsh) => ui.ty.is_ssh(),
            Some(MsgLineCondition::ShowOnlyOnWeb) => ui.ty.is_web(),
            None => true,
        }
    }
}

impl<'a> FromIterator<Span<'a>> for MsgLine {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Span<'a>>,
    {
        Self {
            inner: iter
                .into_iter()
                .map(|span| Span {
                    content: span.content.into_owned().into(),
                    style: span.style,
                })
                .collect(),
            cond: None,
        }
    }
}

impl Styled for MsgLine {
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

#[derive(Clone, Copy, Debug)]
enum MsgLineCondition {
    ShowOnlyOnSsh,
    ShowOnlyOnWeb,
}

#[derive(Clone, Debug)]
pub struct MsgButton<T> {
    inner: Button<'static, T>,
    align: Alignment,
}

impl<T> MsgButton<T> {
    pub fn new(label: impl AsRef<str>, key: KeyCode, resp: T) -> Self {
        Self {
            inner: Button::new(label.as_ref().to_owned(), key).throwing(resp),
            align: Alignment::Left,
        }
    }

    pub fn escape(label: impl AsRef<str>, resp: T) -> Self {
        Self::new(label, KeyCode::Escape, resp)
    }

    pub fn enter(label: impl AsRef<str>, resp: T) -> Self {
        Self::new(label, KeyCode::Enter, resp).right_aligned()
    }

    pub fn right_aligned(mut self) -> Self {
        self.align = Alignment::Right;
        self
    }
}

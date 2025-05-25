use crate::views::game::HelpMsgEvent;
use crate::{Button, FrameType, FromMarkdown, Ui, UiWidget};
use ratatui::layout::Alignment;
use ratatui::style::{Style, Styled};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::Paragraph;
use std::cmp;
use termwiz::input::KeyCode;

#[derive(Clone, Debug)]
pub struct Msg<T = ()> {
    title: &'static str,
    body_ssh: Paragraph<'static>,
    body_web: Paragraph<'static>,
    btns: Vec<MsgBtn<T>>,
}

impl<T> Msg<T> {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(title: &'static str) -> MsgBuilder<T> {
        MsgBuilder {
            title,
            lines: Vec::new(),
            btns: Vec::new(),
        }
    }
}

impl Msg<bool> {
    pub fn start(name: &'static str, docs: &'static [MsgLine]) -> Self {
        Self::new(name)
            .lines(docs.to_vec())
            .btn(MsgBtn::escape("exit", false))
            .btn(MsgBtn::enter("start", true))
            .build()
    }
}

impl Msg<HelpMsgEvent> {
    pub fn help(lines: impl IntoIterator<Item = MsgLine>) -> Self {
        Self::new("help")
            .lines(lines)
            .btn(HelpMsgEvent::close_btn())
            .build()
    }
}

impl<T> UiWidget<T> for &Msg<T>
where
    T: Clone,
{
    type Response = ();

    fn render(self, ui: &mut Ui<T>) -> Self::Response {
        let body = if ui.ty.is_ssh() {
            &self.body_ssh
        } else {
            &self.body_web
        };

        let width = cmp::min(60, ui.area.width - 4);
        let height = body.line_count(width) as u16 + 2;

        ui.imodal(width, height, self.title, |ui| {
            ui.add(body);
            ui.space(height - 1);

            ui.row(|ui| {
                let left_btns =
                    self.btns.iter().filter(|btn| btn.align == Alignment::Left);

                let right_btns = self
                    .btns
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
pub struct MsgBuilder<T> {
    title: &'static str,
    lines: Vec<MsgLine>,
    btns: Vec<MsgBtn<T>>,
}

impl<T> MsgBuilder<T> {
    pub fn line(mut self, line: impl Into<MsgLine>) -> Self {
        self.lines.push(line.into());
        self
    }

    pub fn lines(mut self, lines: impl IntoIterator<Item = MsgLine>) -> Self {
        self.lines.extend(lines);
        self
    }

    pub fn btn(mut self, btn: MsgBtn<T>) -> Self {
        self.btns.push(btn);
        self
    }

    pub fn build(self) -> Msg<T> {
        let body_ssh: Text = self
            .lines
            .iter()
            .filter(|line| line.matches(FrameType::Ssh))
            .map(|line| line.materialize())
            .collect();

        let body_web: Text = self
            .lines
            .iter()
            .filter(|line| line.matches(FrameType::Web))
            .map(|line| line.materialize())
            .collect();

        let body_ssh = Paragraph::new(body_ssh).wrap(Default::default());
        let body_web = Paragraph::new(body_web).wrap(Default::default());

        Msg {
            title: self.title,
            body_ssh,
            body_web,
            btns: self.btns,
        }
    }
}

#[derive(Clone, Debug)]
pub struct MsgLine {
    inner: Line<'static>,
    guard: Option<MsgLineGuard>,
}

impl MsgLine {
    pub fn new(body: impl AsRef<str>) -> Self {
        Self {
            inner: Line::md(body.as_ref()),
            guard: Default::default(),
        }
    }

    /// Creates a line that's visible only for players connected through SSH.
    pub fn ssh(content: impl AsRef<str>) -> Self {
        Self {
            guard: Some(MsgLineGuard::ShowOnlyOnSsh),
            ..Self::new(content)
        }
    }

    /// Creates a line that's visible only for players connected through the
    /// web terminal.
    pub fn web(content: impl AsRef<str>) -> Self {
        Self {
            guard: Some(MsgLineGuard::ShowOnlyOnWeb),
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

    fn matches(&self, ty: FrameType) -> bool {
        match self.guard {
            Some(MsgLineGuard::ShowOnlyOnSsh) => ty.is_ssh(),
            Some(MsgLineGuard::ShowOnlyOnWeb) => ty.is_web(),
            None => true,
        }
    }

    fn materialize(&self) -> Line<'static> {
        self.inner.clone()
    }
}

impl From<&'static str> for MsgLine {
    fn from(value: &'static str) -> Self {
        Self::new(value)
    }
}

impl From<String> for MsgLine {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl<'a> FromIterator<Span<'a>> for MsgLine {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Span<'a>>,
    {
        let inner = iter
            .into_iter()
            .map(|span| Span {
                content: span.content.into_owned().into(),
                style: span.style,
            })
            .collect();

        Self {
            inner,
            guard: Default::default(),
        }
    }
}

impl Styled for MsgLine {
    type Item = Self;

    fn style(&self) -> Style {
        self.inner.style
    }

    fn set_style<S>(mut self, style: S) -> Self::Item
    where
        S: Into<Style>,
    {
        self.inner = self.inner.set_style(style);
        self
    }
}

#[derive(Clone, Copy, Debug)]
enum MsgLineGuard {
    ShowOnlyOnSsh,
    ShowOnlyOnWeb,
}

#[derive(Clone, Debug)]
pub struct MsgBtn<T> {
    inner: Button<'static, T>,
    align: Alignment,
}

impl<T> MsgBtn<T> {
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

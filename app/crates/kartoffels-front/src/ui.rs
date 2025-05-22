use crate::{theme, Button, Clear, FrameType};
use glam::UVec2;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Layout, Position, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Span, Text};
use ratatui::widgets::{Block, Padding, Paragraph, Widget, Wrap};
use std::borrow::Cow;
use termwiz::input::{InputEvent, KeyCode, Modifiers};

#[derive(Debug)]
pub struct Ui<'a, T> {
    pub ty: FrameType,
    pub buf: &'a mut Buffer,
    pub area: Rect,
    pub mouse: Option<&'a (UVec2, bool)>,
    pub event: Option<&'a InputEvent>,
    pub dir: UiDir,
    pub layout: UiLayout,
    pub enabled: bool,
    pub focused: bool,
    pub thrown: Option<T>,
}

impl<T> Ui<'_, T> {
    pub fn with<U>(&mut self, f: impl FnOnce(&mut Ui<T>) -> U) -> U {
        let mut this = Ui {
            ty: self.ty,
            buf: self.buf,
            area: self.area,
            mouse: self.mouse,
            event: self.event,
            dir: self.dir,
            layout: self.layout,
            enabled: self.enabled,
            focused: self.focused,
            thrown: None,
        };

        let result = f(&mut this);

        self.area = this.area;

        if let Some(thrown) = this.thrown {
            self.thrown = Some(thrown);
        }

        result
    }

    pub fn at<U>(&mut self, area: Rect, f: impl FnOnce(&mut Ui<T>) -> U) -> U {
        let old_area = self.area;

        let result = self.with(|this| {
            this.area = this.area.clamp(area);

            f(this)
        });

        self.area = old_area;

        result
    }

    pub fn zoned<U>(&mut self, f: impl FnOnce(&mut Ui<T>) -> U) -> U {
        self.at(self.area, f)
    }

    pub fn enabled<U>(
        &mut self,
        enabled: bool,
        f: impl FnOnce(&mut Ui<T>) -> U,
    ) -> U {
        self.with(|this| {
            this.enabled &= enabled;

            f(this)
        })
    }

    pub fn focused<U>(
        &mut self,
        focused: bool,
        f: impl FnOnce(&mut Ui<T>) -> U,
    ) -> U {
        self.with(|this| {
            this.focused &= focused;

            f(this)
        })
    }

    pub fn add<W>(&mut self, widget: W) -> W::Response
    where
        W: UiWidget<T>,
    {
        widget.render(self)
    }

    pub fn add_at<W>(&mut self, area: Rect, widget: W) -> W::Response
    where
        W: UiWidget<T>,
    {
        self.at(area, |this| this.add(widget))
    }

    pub fn btn<'x>(
        &mut self,
        label: impl Into<Cow<'x, str>>,
        key: impl Into<Option<KeyCode>>,
        f: impl FnOnce(Button<'x, T>) -> Button<'x, T>,
    ) -> <Button<'x, T> as UiWidget<T>>::Response {
        self.add(f(Button::new(label, key)))
    }

    pub fn line<'x>(&mut self, line: impl Into<Text<'x>>) -> u16 {
        let para = Paragraph::new(line).wrap(Wrap::default());
        let height = para.line_count(self.area.width) as u16;

        self.add(para);
        self.space(height);

        height
    }

    pub fn span<'x>(&mut self, span: impl Into<Span<'x>>) {
        let span = span.into();
        let width = span.width() as u16;

        self.add(span);
        self.space(width);
    }

    pub fn block(&mut self, block: Block, f: impl FnOnce(&mut Ui<T>)) {
        self.add(Clear);
        self.add(&block);
        self.at(block.inner(self.area), f);
    }

    fn modal(
        &mut self,
        width: u16,
        height: u16,
        title: Option<&str>,
        border: Color,
        f: impl FnOnce(&mut Ui<T>),
    ) {
        let area = {
            let [_, area, _] = Layout::horizontal([
                Constraint::Fill(1),
                Constraint::Length(width + 4),
                Constraint::Fill(1),
            ])
            .areas(self.area);

            let [_, area, _] = Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(height + 2),
                Constraint::Fill(2),
            ])
            .areas(area);

            area
        };

        self.at(area, |this| {
            let mut block = Block::bordered()
                .border_style(Style::new().fg(border).bg(theme::BG))
                .padding(Padding::horizontal(1));

            if let Some(title) = title {
                block = block.title(title).title_alignment(Alignment::Center);
            }

            this.block(block, f);
        });
    }

    pub fn emodal(
        &mut self,
        width: u16,
        height: u16,
        title: Option<&str>,
        f: impl FnOnce(&mut Ui<T>),
    ) {
        self.modal(width, height, title, theme::RED, f);
    }

    pub fn wmodal(
        &mut self,
        width: u16,
        height: u16,
        title: Option<&str>,
        f: impl FnOnce(&mut Ui<T>),
    ) {
        self.modal(width, height, title, theme::YELLOW, f);
    }

    pub fn imodal(
        &mut self,
        width: u16,
        height: u16,
        title: Option<&str>,
        f: impl FnOnce(&mut Ui<T>),
    ) {
        self.modal(width, height, title, theme::GREEN, f);
    }

    pub fn ltr<U>(&mut self, f: impl FnOnce(&mut Ui<T>) -> U) -> U {
        self.zoned(|this| {
            this.with(|this| {
                this.dir = UiDir::Ltr;

                f(this)
            })
        })
    }

    pub fn rtl<U>(&mut self, f: impl FnOnce(&mut Ui<T>) -> U) -> U {
        self.zoned(|this| {
            this.with(|this| {
                this.dir = UiDir::Rtl;

                f(this)
            })
        })
    }

    pub fn row<U>(&mut self, f: impl FnOnce(&mut Ui<T>) -> U) -> U {
        let result = self.zoned(|this| {
            this.with(|this| {
                this.layout = UiLayout::Row;

                f(this)
            })
        });

        self.space(1);

        result
    }

    pub fn space(&mut self, len: u16) {
        let (off, rem) = match self.layout {
            UiLayout::Row => (&mut self.area.x, &mut self.area.width),
            UiLayout::Col => (&mut self.area.y, &mut self.area.height),
        };

        match self.dir {
            UiDir::Ltr => {
                *off += len;
                *rem -= len;
            }
            UiDir::Rtl => {
                *rem -= len;
            }
        }
    }

    pub fn throw(&mut self, event: T) {
        self.thrown = Some(event);
    }

    pub fn catching<U>(&mut self, f: impl FnOnce(&mut Ui<U>)) -> Option<U> {
        let mut this = Ui {
            ty: self.ty,
            buf: self.buf,
            area: self.area,
            mouse: self.mouse,
            event: self.event,
            dir: self.dir,
            layout: self.layout,
            enabled: self.enabled,
            focused: self.focused,
            thrown: None,
        };

        f(&mut this);

        this.thrown
    }

    pub fn key(&self, key: KeyCode, mods: Modifiers) -> bool {
        if let Some(InputEvent::Key(event)) = &self.event
            && event.key == key
            && event.modifiers == mods
        {
            true
        } else {
            false
        }
    }

    pub fn mouse_pos(&self) -> Option<UVec2> {
        self.mouse.map(|(pos, _)| *pos)
    }

    pub fn mouse_over(&self, area: Rect) -> bool {
        if let Some((pos, _)) = &self.mouse {
            area.contains(Position {
                x: pos.x as u16,
                y: pos.y as u16,
            })
        } else {
            false
        }
    }

    pub fn mouse_pressed(&self) -> bool {
        if let Some((_, pressed)) = &self.mouse {
            *pressed
        } else {
            false
        }
    }
}

/// Directions of widgets within the interface.
///
/// This affects only the high-level layout - not, say, direction of characters
/// within a paragraph.
#[derive(Clone, Copy, Debug)]
pub enum UiDir {
    /// Left-to-right
    Ltr,

    /// Right-to-left
    Rtl,
}

impl From<UiDir> for Alignment {
    fn from(dir: UiDir) -> Self {
        match dir {
            UiDir::Ltr => Alignment::Left,
            UiDir::Rtl => Alignment::Right,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum UiLayout {
    Row,
    Col,
}

impl UiLayout {
    pub fn is_row(&self) -> bool {
        matches!(self, UiLayout::Row)
    }

    pub fn is_col(&self) -> bool {
        matches!(self, UiLayout::Col)
    }
}

pub trait UiWidget<T> {
    type Response = ();

    fn render(self, ui: &mut Ui<T>) -> Self::Response;
}

impl<T, W> UiWidget<T> for W
where
    W: Widget,
{
    fn render(self, ui: &mut Ui<T>) -> Self::Response {
        W::render(self, ui.area, ui.buf)
    }
}

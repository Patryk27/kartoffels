use crate::{theme, Clear, TermEndpoint};
use glam::UVec2;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Layout, Position, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Span, Text};
use ratatui::widgets::{Block, Padding, Paragraph, Widget, WidgetRef, Wrap};
use termwiz::input::{InputEvent, KeyCode, Modifiers};

#[derive(Debug)]
pub struct Ui<'a, T> {
    pub endpoint: TermEndpoint,
    pub buf: &'a mut Buffer,
    pub area: Rect,
    pub(super) mouse: Option<&'a (UVec2, bool)>,
    pub event: Option<&'a InputEvent>,
    pub(super) clipboard: &'a mut Vec<String>,
    pub layout: UiLayout,
    pub enabled: bool,
    pub(super) thrown: &'a mut Option<T>,
}

impl<T> Ui<'_, T> {
    pub fn with<U>(&mut self, f: impl FnOnce(&mut Ui<T>) -> U) -> U {
        f(&mut Ui {
            endpoint: self.endpoint,
            buf: self.buf,
            area: self.area,
            mouse: self.mouse,
            event: self.event,
            clipboard: self.clipboard,
            layout: self.layout,
            enabled: self.enabled,
            thrown: self.thrown,
        })
    }

    pub fn clamp<U>(
        &mut self,
        area: Rect,
        f: impl FnOnce(&mut Ui<T>) -> U,
    ) -> U {
        self.with(|ui| {
            ui.area = ui.area.clamp(area);

            f(ui)
        })
    }

    pub fn row<U>(&mut self, f: impl FnOnce(&mut Ui<T>) -> U) -> U {
        self.with(|ui| {
            ui.layout = UiLayout::Row;

            f(ui)
        })
    }

    pub fn enable<U>(
        &mut self,
        enabled: bool,
        f: impl FnOnce(&mut Ui<T>) -> U,
    ) -> U {
        self.with(|ui| {
            ui.enabled = ui.enabled && enabled;

            f(ui)
        })
    }

    pub fn space(&mut self, len: u16) {
        match self.layout {
            UiLayout::Row => {
                self.area.x += len;
                self.area.width -= len;
            }

            UiLayout::Col => {
                self.area.y += len;
                self.area.height -= len;
            }
        }
    }

    pub fn line<'x>(&mut self, line: impl Into<Text<'x>>) -> u16 {
        let para = Paragraph::new(line).wrap(Wrap::default());
        let height = para.line_count(self.area.width) as u16;

        para.render(self.area, self.buf);
        self.space(height);

        height
    }

    pub fn span<'x>(&mut self, span: impl Into<Span<'x>>) {
        let span = span.into();
        let width = span.width() as u16;

        span.render(self.area, self.buf);

        self.space(width);
    }

    pub fn block(&mut self, block: Block, f: impl FnOnce(&mut Ui<T>)) {
        Clear::render(self);
        block.render_ref(self.area, self.buf);

        self.clamp(block.inner(self.area), f);
    }

    pub fn window(
        &mut self,
        width: u16,
        height: u16,
        title: Option<&str>,
        border_fg: Color,
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

        self.clamp(area, |ui| {
            let mut block = Block::bordered()
                .border_style(Style::new().fg(border_fg).bg(theme::BG))
                .padding(Padding::horizontal(1));

            if let Some(title) = title {
                block = block.title(title).title_alignment(Alignment::Center);
            }

            ui.block(block, f);
        });
    }

    pub fn info_window(
        &mut self,
        width: u16,
        height: u16,
        title: Option<&str>,
        f: impl FnOnce(&mut Ui<T>),
    ) {
        self.window(width, height, title, theme::GREEN, f);
    }

    pub fn error_window(
        &mut self,
        width: u16,
        height: u16,
        title: Option<&str>,
        f: impl FnOnce(&mut Ui<T>),
    ) {
        self.window(width, height, title, theme::RED, f);
    }

    pub fn key(&self, key: KeyCode, mods: Modifiers) -> bool {
        if let Some(InputEvent::Key(event)) = &self.event {
            if event.key == key && event.modifiers == mods {
                return true;
            }
        }

        false
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

    pub fn copy(&mut self, payload: impl AsRef<str>) {
        self.clipboard.push(payload.as_ref().to_owned());
    }

    pub fn throw(&mut self, event: T) {
        *self.thrown = Some(event);
    }

    pub fn catch<U>(&mut self, f: impl FnOnce(&mut Ui<U>)) -> Option<U> {
        let mut thrown = None;

        f(&mut Ui {
            endpoint: self.endpoint,
            buf: self.buf,
            area: self.area,
            mouse: self.mouse,
            event: self.event,
            clipboard: self.clipboard,
            layout: self.layout,
            enabled: self.enabled,
            thrown: &mut thrown,
        });

        thrown
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

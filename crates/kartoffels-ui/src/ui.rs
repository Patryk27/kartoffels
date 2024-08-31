use crate::{theme, Clear, RectExt};
use glam::UVec2;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Layout, Position, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Text;
use ratatui::widgets::{Block, Padding, Widget};
use ratatui::Frame;
use std::future::Future;
use std::pin::pin;
use std::task::{Context, Poll, Waker};
use termwiz::input::{InputEvent, KeyCode, Modifiers, MouseButtons};
use tokio::time::Interval;

#[derive(Debug)]
pub struct Ui<'a, 'b> {
    waker: &'a Waker,
    frame: &'a mut Frame<'b>,
    area: Rect,
    mouse: Option<(UVec2, MouseButtons)>,
    event: Option<&'a InputEvent>,
    layout: UiLayout,
}

impl<'a, 'b> Ui<'a, 'b> {
    pub fn new(
        waker: &'a Waker,
        frame: &'a mut Frame<'b>,
        mouse: Option<(UVec2, MouseButtons)>,
        event: Option<&'a InputEvent>,
    ) -> Self {
        let area = frame.area();

        Self {
            waker,
            frame,
            area,
            mouse,
            event,
            layout: UiLayout::Col,
        }
    }

    pub fn buf(&mut self) -> &mut Buffer {
        self.frame.buffer_mut()
    }

    pub fn area(&self) -> Rect {
        self.area
    }

    pub fn event(&self) -> Option<&InputEvent> {
        self.event
    }

    pub fn layout(&self) -> UiLayout {
        self.layout
    }

    pub fn clamp<T>(&mut self, area: Rect, f: impl FnOnce(&mut Ui) -> T) -> T {
        f(&mut Ui {
            waker: self.waker,
            frame: self.frame,
            area: self.area.clamp(area),
            mouse: self.mouse.clone(),
            event: self.event,
            layout: self.layout,
        })
    }

    pub fn row<T>(&mut self, f: impl FnOnce(&mut Ui) -> T) -> T {
        self.clamp(self.area.footer(), |ui| {
            ui.layout = UiLayout::Row;

            f(ui)
        })
    }

    pub fn step(&mut self, len: u16) {
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

    pub fn line<'x>(&mut self, text: impl Into<Text<'x>>) {
        self.text(text);
        self.step(1);
    }

    pub fn text<'x>(&mut self, text: impl Into<Text<'x>>) {
        text.into().render(self.area, self.buf());
    }

    pub fn dialog<T>(
        &mut self,
        width: u16,
        height: u16,
        title: Option<&str>,
        border_fg: Color,
        f: impl FnOnce(&mut Ui) -> T,
    ) -> T {
        let area = {
            let [_, area, _] = Layout::horizontal([
                Constraint::Fill(1),
                Constraint::Length(width + 4),
                Constraint::Fill(1),
            ])
            .areas(self.area());

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

            let inner_area = block.inner(ui.area());

            Clear::render(ui);
            block.render(ui.area(), ui.buf());
            ui.clamp(inner_area, f)
        })
    }

    pub fn info_dialog<T>(
        &mut self,
        width: u16,
        height: u16,
        title: Option<&str>,
        f: impl FnOnce(&mut Ui) -> T,
    ) -> T {
        self.dialog(width, height, title, theme::GREEN, f)
    }

    pub fn error_dialog<T>(
        &mut self,
        width: u16,
        height: u16,
        title: Option<&str>,
        f: impl FnOnce(&mut Ui) -> T,
    ) -> T {
        self.dialog(width, height, title, theme::RED, f)
    }

    pub fn key(&self, key: KeyCode, mods: Modifiers) -> bool {
        if let Some(InputEvent::Key(event)) = &self.event {
            if event.key == key && event.modifiers == mods {
                return true;
            }
        }

        false
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

    // TODO: prevent double-press
    pub fn mouse_pressed(&self) -> bool {
        if let Some((_, btns)) = &self.mouse {
            btns.contains(MouseButtons::LEFT)
        } else {
            false
        }
    }

    pub fn poll<F>(&mut self, mut f: F) -> Poll<F::Output>
    where
        F: Future,
    {
        pin!(f).poll(&mut Context::from_waker(self.waker))
    }

    pub fn poll_interval(&mut self, int: &mut Interval) -> bool {
        if self.poll(int.tick()).is_ready() {
            // Tokio's intervals don't reschedule themselves upon a completed
            // tick - we have to do it by hand:
            _ = self.poll(int.tick());

            true
        } else {
            false
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
}

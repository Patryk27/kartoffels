use crate::{theme, Clear, LayoutExt, RectExt};
use glam::UVec2;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Padding, Widget};
use ratatui::Frame;
use std::future::Future;
use std::pin::pin;
use std::task::{Context, Poll, Waker};
use termwiz::input::{InputEvent, KeyCode, Modifiers, MouseButtons};

#[derive(Debug)]
pub struct Ui<'a, 'b> {
    waker: &'a Waker,
    frame: &'a mut Frame<'b>,
    area: Rect,
    mouse: Option<(UVec2, MouseButtons)>,
    event: Option<InputEvent>,
    layout: UiLayout,
}

impl<'a, 'b> Ui<'a, 'b> {
    pub fn new(
        waker: &'a Waker,
        frame: &'a mut Frame<'b>,
        mouse: Option<(UVec2, MouseButtons)>,
        event: Option<InputEvent>,
    ) -> Self {
        let area = frame.area();

        Self {
            waker,
            frame,
            area,
            mouse,
            event,
            layout: UiLayout::Row,
        }
    }

    pub fn buf(&mut self) -> &mut Buffer {
        self.frame.buffer_mut()
    }

    pub fn area(&self) -> Rect {
        self.area
    }

    pub fn event(&self) -> Option<&InputEvent> {
        self.event.as_ref()
    }

    pub fn clamp<T>(&mut self, area: Rect, f: impl FnOnce(&mut Ui) -> T) -> T {
        f(&mut Ui {
            waker: self.waker,
            frame: self.frame,
            area: self.area.clamp(area),
            mouse: self.mouse.clone(),
            event: self.event.clone(), // TODO expensive
            layout: self.layout,
        })
    }

    pub fn dialog<T>(
        &mut self,
        width: u16,
        height: u16,
        title: Option<&str>,
        border_fg: Color,
        f: impl FnOnce(&mut Ui) -> T,
    ) -> T {
        let area = Layout::dialog(width, height, self.area);

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

    pub fn row<T>(&mut self, f: impl FnOnce(&mut Ui) -> T) -> T {
        self.clamp(self.area.footer(), |ui| {
            ui.layout = UiLayout::Row;

            f(ui)
        })
    }

    pub fn step(&mut self, len: u16) {
        match self.layout {
            UiLayout::Row => {
                self.area.y += len;
                self.area.height -= len;
            }

            UiLayout::Col => {
                self.area.x += len;
                self.area.width -= len;
            }
        }
    }

    pub fn key(&self, key: KeyCode, mods: Modifiers) -> bool {
        if let Some(InputEvent::Key(event)) = &self.event {
            if event.key == key && event.modifiers == mods {
                return true;
            }
        }

        false
    }

    pub fn poll<F>(&mut self, mut f: F) -> Poll<F::Output>
    where
        F: Future,
    {
        pin!(f).poll(&mut Context::from_waker(self.waker))
    }
}

#[derive(Clone, Copy, Debug)]
enum UiLayout {
    Row,
    Col,
}

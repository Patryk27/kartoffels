#![feature(associated_type_defaults)]
#![feature(let_chains)]
#![feature(str_as_str)]

mod abort;
mod compat;
mod ui;
mod utils;
mod widgets;

pub mod theme;

pub use self::abort::*;
pub use self::ui::*;
pub use self::utils::*;
pub use self::widgets::*;
pub use termwiz::input::{InputEvent, KeyCode, Modifiers};

use self::compat::*;
use anyhow::{Context, Error, Result};
use glam::{uvec2, UVec2};
use ratatui::crossterm::event::{
    DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste,
    EnableMouseCapture,
};
use ratatui::crossterm::terminal::{
    self, EnterAlternateScreen, LeaveAlternateScreen, SetTitle,
};
use ratatui::crossterm::{cursor, Command};
use ratatui::layout::Rect;
use ratatui::prelude::CrosstermBackend;
use ratatui::widgets::{Paragraph, Widget};
use ratatui::{Terminal, TerminalOptions, Viewport};
use std::mem;
use termwiz::escape::osc::Selection;
use termwiz::escape::OperatingSystemCommand;
use termwiz::input::{InputParser, MouseButtons, MouseEvent};
use tokio::sync::mpsc;
use tokio::time::Interval;
use tokio::{select, time};
use tracing::warn;

pub type Stdin = mpsc::Receiver<Vec<u8>>;
pub type Stdout = mpsc::Sender<Vec<u8>>;

#[derive(Debug)]
pub struct Frame {
    ty: FrameType,
    size: UVec2,
    stdin: Stdin,
    stdout: Stdout,
    term: Terminal<CrosstermBackend<WriterProxy>>,
    parser: InputParser,
    mouse: FrameMouse,
    event: Option<InputEvent>,
    frames: Interval,
}

impl Frame {
    /// Opcode used to inform the frame that the underlying terminal has been
    /// resized.
    ///
    /// Web terminal emits this code natively, while for the SSH terminal it's
    /// the Rust backend itself which sorta "injects" this instruction into the
    /// stdin.
    ///
    /// Overall, this is kind of a hack - ideally we'd have something like:
    ///
    /// ```no_run
    /// enum StdinOpcode {
    ///     Input(Vec<u8>),
    ///     Resized(u8, u8),
    /// }
    /// ```
    ///
    /// ... but transmitting such enum efficiently over web sockets is awkward.
    ///
    /// As for the value, 0x04 has been chosen with a fair dice roll - we just
    /// never expect to stumble upon this value during normal communication.
    pub const CMD_RESIZE: u8 = 0x04;

    /// Minimum size of the frame.
    ///
    /// Chosen out of practicality - designing UI for any arbitrary size would
    /// be impossible.
    pub const MIN_SIZE: UVec2 = uvec2(80, 30);

    /// Maximum size of the frame.
    ///
    /// Chosen out of practicaly as well - larger viewports take more resources
    /// to handle, it just doesn't scale that well.
    pub const MAX_SIZE: UVec2 = uvec2(160, 60);

    pub fn new(
        ty: FrameType,
        size: UVec2,
        stdin: Stdin,
        stdout: Stdout,
    ) -> Result<Self> {
        let size = size.min(Self::MAX_SIZE);

        let mut term = {
            let writer = WriterProxy::default();
            let backend = CrosstermBackend::new(writer);

            let opts = TerminalOptions {
                viewport: Viewport::Fixed(Self::viewport_rect(size)),
            };

            Terminal::with_options(backend, opts)?
        };

        term.clear()?;

        Ok(Self {
            ty,
            size,
            stdin,
            stdout,
            term,
            parser: Default::default(),
            mouse: Default::default(),
            event: Default::default(),
            frames: time::interval(theme::FRAME_TIME),
        })
    }

    pub fn ty(&self) -> FrameType {
        self.ty
    }

    pub fn size(&self) -> UVec2 {
        self.size
    }

    pub async fn init(&mut self) -> Result<()> {
        let mut cmds = String::new();

        _ = EnterAlternateScreen.write_ansi(&mut cmds);
        _ = EnableBracketedPaste.write_ansi(&mut cmds);
        _ = EnableMouseCapture.write_ansi(&mut cmds);
        _ = SetTitle("kartoffels").write_ansi(&mut cmds);

        self.send(cmds.into()).await?;

        Ok(())
    }

    pub async fn destroy(&mut self) -> Result<()> {
        let mut cmds = String::new();

        match self.ty {
            FrameType::Ssh => {
                _ = DisableMouseCapture.write_ansi(&mut cmds);
                _ = DisableBracketedPaste.write_ansi(&mut cmds);
                _ = LeaveAlternateScreen.write_ansi(&mut cmds);
                _ = cursor::Show.write_ansi(&mut cmds);
            }

            FrameType::Web => {
                _ = terminal::Clear(terminal::ClearType::All)
                    .write_ansi(&mut cmds);

                _ = cursor::MoveTo(0, 0).write_ansi(&mut cmds);
            }
        }

        self.send(cmds.into()).await?;

        Ok(())
    }

    pub async fn update<F, T>(&mut self, render: F) -> Result<Option<T>>
    where
        F: FnOnce(&mut Ui<T>),
    {
        let result = self.draw(render).await?;

        self.sleep().await?;

        Ok(result)
    }

    async fn draw<F, T>(&mut self, render: F) -> Result<Option<T>>
    where
        F: FnOnce(&mut Ui<T>),
    {
        let mut event = None;

        if self.size.cmplt(Self::MIN_SIZE).any() {
            self.term.draw(|frame| {
                let area = frame.area();
                let buf = frame.buffer_mut();

                Clear::render_ex(area, buf);

                let msg = Paragraph::new(
                    "ouch, your terminal is too small to play kartoffels - try \
                     zooming out (Ctrl+-, Cmd+- etc.)",
                )
                .wrap(Default::default());

                Widget::render(msg, area, buf);
            })?;
        } else {
            self.term.draw(|frame| {
                let area = frame.area();

                render(&mut Ui {
                    ty: self.ty,
                    buf: frame.buffer_mut(),
                    area,
                    mouse: self.mouse.report().as_ref(),
                    event: self.event.take().as_ref(),
                    layout: UiLayout::Col,
                    enabled: true,
                    thrown: &mut event,
                });
            })?;
        }

        self.flush().await?;

        Ok(event)
    }

    async fn sleep(&mut self) -> Result<()> {
        select! {
            stdin = self.stdin.recv() => {
                // After retrieving an input event, reset the "when next frame"
                // interval, so that we don't refresh the interface needlessly.
                //
                // The reasoning here goes: since the user will already get a
                // brand new frame after pressing a keystroke, we're covered for
                // the next `FRAME_TIME` milliseconds anyway.
                self.frames.reset();
                self.handle(stdin.context("lost stdin")?)?;
            },

            _ = self.frames.tick() => {
                // Just a wake-up, so that we can refresh the interface
            },
        }

        Ok(())
    }

    fn handle(&mut self, stdin: Vec<u8>) -> Result<()> {
        if let Some(stdin) = stdin.strip_prefix(&[Self::CMD_RESIZE]) {
            self.handle_resize(stdin)?;
        } else {
            self.handle_input(stdin)?;
        }

        Ok(())
    }

    fn handle_resize(&mut self, stdin: &[u8]) -> Result<()> {
        let cols = stdin.first().copied().unwrap_or(0);
        let rows = stdin.last().copied().unwrap_or(0);

        self.size = uvec2(cols as u32, rows as u32).min(Self::MAX_SIZE);
        self.term.resize(Self::viewport_rect(self.size))?;

        Ok(())
    }

    fn handle_input(&mut self, stdin: Vec<u8>) -> Result<()> {
        let events = self.parser.parse_as_vec(&stdin, false);

        for event in events {
            if let InputEvent::Key(event) = &event {
                match (event.key, event.modifiers) {
                    Abort::SOFT_BINDING => {
                        return Err(Error::new(Abort { soft: true }));
                    }

                    Abort::HARD_BINDING if self.ty.is_ssh() => {
                        return Err(Error::new(Abort { soft: false }));
                    }

                    _ => (),
                }
            }

            match event {
                InputEvent::Mouse(event) => {
                    self.mouse.update(event);
                }

                event => {
                    if self.event.is_some() {
                        warn!("missed event: {:?}", self.event);
                    }

                    self.event = Some(event);
                }
            }
        }

        Ok(())
    }

    async fn flush(&mut self) -> Result<()> {
        let writer = self.term.backend_mut().writer_mut();

        if writer.flushed {
            writer.flushed = false;

            self.stdout.send(mem::take(&mut writer.buffer)).await?;
        }

        Ok(())
    }

    fn viewport_rect(size: UVec2) -> Rect {
        Rect {
            x: 0,
            y: 0,
            width: size.x.min(255) as u16,
            height: size.y.min(255) as u16,
        }
    }

    pub async fn copy(&mut self, payload: String) -> Result<()> {
        let cmd =
            OperatingSystemCommand::SetSelection(Selection::CLIPBOARD, payload)
                .to_string()
                .into_bytes();

        self.send(cmd).await?;

        Ok(())
    }

    pub async fn send(&mut self, stdout: Vec<u8>) -> Result<()> {
        self.stdout.send(stdout).await?;

        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub enum FrameType {
    Ssh,
    Web,
}

impl FrameType {
    pub fn is_ssh(&self) -> bool {
        matches!(self, Self::Ssh)
    }

    pub fn is_web(&self) -> bool {
        matches!(self, Self::Web)
    }
}

#[derive(Clone, Debug, Default)]
struct FrameMouse {
    pos: Option<UVec2>,
    click: FrameMouseClick,
}

impl FrameMouse {
    fn update(&mut self, event: MouseEvent) {
        self.pos = Some(uvec2(event.x as u32, event.y as u32) - 1);

        if event.mouse_buttons.contains(MouseButtons::LEFT) {
            self.click = match self.click {
                FrameMouseClick::NotClicked => {
                    FrameMouseClick::ClickedButNotReported
                }
                click => click,
            };
        } else {
            self.click = FrameMouseClick::NotClicked;
        }
    }

    fn report(&mut self) -> Option<(UVec2, bool)> {
        let pos = self.pos?;

        let clicked =
            matches!(self.click, FrameMouseClick::ClickedButNotReported);

        if clicked {
            self.click = FrameMouseClick::ClickedAndReported;
        }

        Some((pos, clicked))
    }
}

#[derive(Clone, Copy, Debug, Default)]
enum FrameMouseClick {
    #[default]
    NotClicked,
    ClickedButNotReported,
    ClickedAndReported,
}

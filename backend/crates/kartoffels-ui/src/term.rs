use crate::{theme, Abort, Clear, Ui, UiLayout};
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
use std::io::{self, Write};
use std::mem;
use termwiz::escape::osc::Selection;
use termwiz::escape::OperatingSystemCommand;
use termwiz::input::{InputEvent, InputParser, MouseButtons, MouseEvent};
use tokio::sync::mpsc;
use tokio::{select, time};
use tracing::warn;

pub type Stdin = mpsc::Receiver<Vec<u8>>;
pub type Stdout = mpsc::Sender<Vec<u8>>;

#[derive(Debug)]
pub struct Term {
    ty: TermType,
    stdin: Stdin,
    stdin_parser: InputParser,
    stdout: Stdout,
    term: Terminal<CrosstermBackend<WriterProxy>>,
    size: UVec2,
    mouse: TermMouse,
    event: Option<InputEvent>,
    frames: time::Interval,
}

impl Term {
    pub const CMD_RESIZE: u8 = 0x04;

    pub fn new(
        ty: TermType,
        stdin: Stdin,
        stdout: Stdout,
        size: UVec2,
    ) -> Result<Self> {
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
            stdin,
            stdin_parser: Default::default(),
            stdout,
            term,
            size,
            mouse: Default::default(),
            event: Default::default(),
            frames: time::interval(theme::FRAME_TIME),
        })
    }

    pub fn ty(&self) -> TermType {
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

    pub async fn finalize(&mut self) -> Result<()> {
        let mut cmds = String::new();

        match self.ty {
            TermType::Ssh => {
                _ = DisableMouseCapture.write_ansi(&mut cmds);
                _ = DisableBracketedPaste.write_ansi(&mut cmds);
                _ = LeaveAlternateScreen.write_ansi(&mut cmds);
                _ = cursor::Show.write_ansi(&mut cmds);
            }

            TermType::Web => {
                _ = terminal::Clear(terminal::ClearType::All)
                    .write_ansi(&mut cmds);

                _ = cursor::MoveTo(0, 0).write_ansi(&mut cmds);
            }
        }

        self.send(cmds.into()).await?;

        Ok(())
    }

    pub async fn draw<F, T>(&mut self, render: F) -> Result<Option<T>>
    where
        F: FnOnce(&mut Ui) -> T,
    {
        let mut resp = None;
        let mut clipboard = Vec::new();

        if self.size.x < 50 || self.size.y < 30 {
            self.term.draw(|frame| {
                let area = frame.area();
                let buf = frame.buffer_mut();

                Clear::render_ex(area, buf);

                Paragraph::new(
                    "whoopsie, your terminal is too small to play kartoffels\
                     \n\n\
                     buy something with at least 50x30 characters",
                )
                .wrap(Default::default())
                .render(area, buf);
            })?;
        } else {
            self.term.draw(|frame| {
                let area = frame.area();

                resp = Some(render(&mut Ui {
                    ty: self.ty,
                    frame,
                    area,
                    mouse: self.mouse.report().as_ref(),
                    event: self.event.take().as_ref(),
                    clipboard: &mut clipboard,
                    layout: UiLayout::Col,
                    enabled: true,
                    thrown: &mut None,
                }));
            })?;
        }

        self.flush().await?;

        for payload in clipboard {
            self.copy_to_clipboard(payload).await?;
        }

        Ok(resp)
    }

    pub async fn poll(&mut self) -> Result<()> {
        select! {
            stdin = self.stdin.recv() => {
                // After retrieving an input event, reset the "when next frame"
                // interval, so that we don't refresh the interface needlessly.
                //
                // The reasoning here goes: since the user will already get a
                // brand new frame after pressing a keystroke, we're covered for
                // the next `FRAME_TIME` milliseconds anyway.
                self.frames.reset();
                self.recv(stdin.context("lost stdin")?)?;
            },

            _ = self.frames.tick() => {
                // Just a wake-up, so that we can refresh the interface
            },
        }

        Ok(())
    }

    pub async fn copy_to_clipboard(&mut self, payload: String) -> Result<()> {
        let cmd =
            OperatingSystemCommand::SetSelection(Selection::CLIPBOARD, payload)
                .to_string()
                .into_bytes();

        self.send(cmd).await?;

        Ok(())
    }

    fn recv(&mut self, stdin: Vec<u8>) -> Result<()> {
        if let Some(size) = stdin.strip_prefix(&[Self::CMD_RESIZE]) {
            let cols = size.first().copied().unwrap_or(0);
            let rows = size.last().copied().unwrap_or(0);

            self.size = uvec2(cols as u32, rows as u32);
            self.term.resize(Self::viewport_rect(self.size))?;
        } else {
            let events = self.stdin_parser.parse_as_vec(&stdin, false);

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
        }

        Ok(())
    }

    pub async fn send(&mut self, stdout: Vec<u8>) -> Result<()> {
        self.stdout.send(stdout).await?;

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
}

#[derive(Clone, Copy, Debug)]
pub enum TermType {
    Ssh,
    Web,
}

impl TermType {
    pub fn is_ssh(&self) -> bool {
        matches!(self, TermType::Ssh)
    }

    pub fn is_web(&self) -> bool {
        matches!(self, TermType::Web)
    }
}

#[derive(Default, Debug)]
struct WriterProxy {
    buffer: Vec<u8>,
    flushed: bool,
}

impl Write for WriterProxy {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buffer.extend(buf);

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.flushed = true;

        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
struct TermMouse {
    pos: Option<UVec2>,
    click: TermMouseClick,
}

impl TermMouse {
    fn update(&mut self, event: MouseEvent) {
        self.pos = Some(uvec2(event.x as u32, event.y as u32) - 1);

        if event.mouse_buttons.contains(MouseButtons::LEFT) {
            self.click = match self.click {
                TermMouseClick::NotClicked => {
                    TermMouseClick::ClickedButNotReported
                }
                click => click,
            };
        } else {
            self.click = TermMouseClick::NotClicked;
        }
    }

    fn report(&mut self) -> Option<(UVec2, bool)> {
        let pos = self.pos?;

        let clicked =
            matches!(self.click, TermMouseClick::ClickedButNotReported);

        if clicked {
            self.click = TermMouseClick::ClickedAndReported;
        }

        Some((pos, clicked))
    }
}

#[derive(Clone, Copy, Debug, Default)]
enum TermMouseClick {
    #[default]
    NotClicked,
    ClickedButNotReported,
    ClickedAndReported,
}

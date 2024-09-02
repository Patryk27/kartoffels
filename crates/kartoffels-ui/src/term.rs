use crate::Ui;
use anyhow::{anyhow, Error, Result};
use futures_util::{Sink, SinkExt, Stream, StreamExt};
use glam::{uvec2, UVec2};
use ratatui::crossterm::event::{
    DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste,
    EnableMouseCapture,
};
use ratatui::crossterm::terminal::{
    self, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::crossterm::{cursor, Command};
use ratatui::layout::Rect;
use ratatui::prelude::CrosstermBackend;
use ratatui::{Terminal, TerminalOptions, Viewport};
use std::io::{self, Write};
use std::mem;
use std::pin::Pin;
use std::sync::Arc;
use termwiz::input::{
    InputEvent, InputParser, KeyCode, Modifiers, MouseButtons,
};
use tokio::select;
use tokio::sync::Notify;

pub type Stdin = Pin<Box<dyn Stream<Item = Result<Vec<u8>>> + Send + Sync>>;
pub type Stdout = Pin<Box<dyn Sink<Vec<u8>, Error = Error> + Send + Sync>>;

pub struct Term {
    ty: TermType,
    stdin: Stdin,
    stdin_parser: InputParser,
    stdout: Stdout,
    term: Terminal<CrosstermBackend<WriterProxy>>,
    size: UVec2,
    mouse: Option<(UVec2, MouseButtons)>,
    event: Option<InputEvent>,
    notify: Arc<Notify>,
}

impl Term {
    pub async fn new(
        ty: TermType,
        stdin: Stdin,
        stdout: Stdout,
        size: UVec2,
    ) -> Result<Self> {
        let stdin = Box::pin(stdin);
        let mut stdout = Box::pin(stdout);

        let mut term = {
            let writer = WriterProxy::default();
            let backend = CrosstermBackend::new(writer);

            let opts = TerminalOptions {
                viewport: Viewport::Fixed(Self::viewport_rect(size)),
            };

            Terminal::with_options(backend, opts)?
        };

        term.clear()?;
        stdout.send(Self::enter_cmds().into_bytes()).await?;

        Ok(Self {
            ty,
            stdin,
            stdin_parser: InputParser::new(),
            stdout,
            term,
            size,
            mouse: None,
            event: None,
            notify: Arc::new(Notify::new()),
        })
    }

    pub fn enter_cmds() -> String {
        let mut cmds = String::new();

        _ = EnterAlternateScreen.write_ansi(&mut cmds);
        _ = EnableBracketedPaste.write_ansi(&mut cmds);
        _ = EnableMouseCapture.write_ansi(&mut cmds);

        cmds
    }

    pub fn leave_cmds() -> String {
        let mut cmds = String::new();

        _ = DisableMouseCapture.write_ansi(&mut cmds);
        _ = DisableBracketedPaste.write_ansi(&mut cmds);
        _ = LeaveAlternateScreen.write_ansi(&mut cmds);
        _ = cursor::Show.write_ansi(&mut cmds);

        cmds
    }

    pub fn reset_cmds() -> Vec<u8> {
        let mut cmd = String::new();

        _ = terminal::Clear(terminal::ClearType::All).write_ansi(&mut cmd);
        _ = cursor::MoveTo(0, 0).write_ansi(&mut cmd);

        cmd.into()
    }

    pub fn crashed_msg() -> Vec<u8> {
        "whoopsie, the game has crashed!\r\n".into()
    }

    pub fn shutting_down_msg() -> Vec<u8> {
        "whoopsie, the server is shutting down!\r\n".into()
    }

    pub fn ty(&self) -> TermType {
        self.ty
    }

    pub fn size(&self) -> UVec2 {
        self.size
    }

    pub async fn draw<F, T>(&mut self, render: F) -> Result<T>
    where
        F: FnOnce(&mut Ui) -> T,
    {
        // TODO constructing waker once should be enough
        let waker = {
            let notify = self.notify.clone();

            waker_fn::waker_fn(move || {
                notify.notify_waiters();
            })
        };

        let mut result = None;

        self.term.draw(|frame| {
            result = Some(render(&mut Ui::new(
                &waker,
                frame,
                self.mouse.clone(),
                self.event.take().as_ref(),
                self.ty,
            )));
        })?;

        self.flush().await?;

        Ok(result.unwrap())
    }

    pub async fn tick(&mut self) -> Result<()> {
        let bytes = select! {
            bytes = self.stdin.next() => Some(bytes),
            _ = self.notify.notified() => None,
        };

        if let Some(bytes) = bytes {
            let bytes = bytes.ok_or_else(|| anyhow!("lost stdin"))??;

            if let Some(size) = bytes.strip_prefix(&[0x04]) {
                let cols = size.first().copied().unwrap_or(0);
                let rows = size.last().copied().unwrap_or(0);

                self.size = uvec2(cols as u32, rows as u32);
                self.term.resize(Self::viewport_rect(self.size))?;
            } else {
                let events = self.stdin_parser.parse_as_vec(&bytes, false);

                for event in events {
                    if let InputEvent::Key(event) = &event {
                        if event.key == KeyCode::Char('c')
                            && event.modifiers == Modifiers::CTRL
                        {
                            return Err(anyhow!("got C-c"));
                        }
                    }

                    match event {
                        InputEvent::Mouse(event) => {
                            self.mouse = Some((
                                uvec2(event.x as u32 - 1, event.y as u32 - 1),
                                event.mouse_buttons.clone(),
                            ));
                        }

                        event => {
                            self.event = Some(event);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn send(&mut self, data: Vec<u8>) -> Result<()> {
        self.stdout.send(data).await?;

        Ok(())
    }

    async fn flush(&mut self) -> Result<()> {
        let writer = self.term.backend_mut().writer_mut();

        if writer.flushed {
            writer.flushed = false;

            // TODO could be more efficient using async mutex
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
    Http,
    Ssh,
}

impl TermType {
    pub fn is_http(&self) -> bool {
        matches!(self, TermType::Http)
    }

    pub fn is_ssh(&self) -> bool {
        matches!(self, TermType::Ssh)
    }
}

#[derive(Default)]
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

use crate::Ui;
use anyhow::{anyhow, Error, Result};
use futures_util::{Sink, SinkExt, Stream, StreamExt};
use glam::{uvec2, UVec2};
use ratatui::crossterm::event::{
    DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste,
    EnableMouseCapture,
};
use ratatui::crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen,
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
    stdin: Stdin,
    stdin_parser: InputParser,
    stdout: Stdout,
    term: Terminal<CrosstermBackend<WriterProxy>>,
    size: UVec2,
    mouse: Option<(UVec2, MouseButtons)>,
    event: Option<InputEvent>,
    notify: Arc<Notify>,
    initialized: bool,
}

impl Term {
    pub fn new(stdin: Stdin, stdout: Stdout, size: UVec2) -> Result<Self> {
        let stdin = Box::pin(stdin);
        let stdout = Box::pin(stdout);

        let term = {
            let writer = WriterProxy::default();
            let backend = CrosstermBackend::new(writer);

            let opts = TerminalOptions {
                viewport: Viewport::Fixed(Self::viewport_rect(size)),
            };

            Terminal::with_options(backend, opts)?
        };

        Ok(Self {
            stdin,
            stdin_parser: InputParser::new(),
            stdout,
            term,
            size,
            mouse: None,
            event: None,
            notify: Arc::new(Notify::new()),
            initialized: false,
        })
    }

    pub fn enter_sequence() -> String {
        let mut cmds = String::new();

        _ = EnterAlternateScreen.write_ansi(&mut cmds);
        _ = EnableBracketedPaste.write_ansi(&mut cmds);
        _ = EnableMouseCapture.write_ansi(&mut cmds);

        cmds
    }

    pub fn leave_sequence() -> String {
        let mut cmds = String::new();

        _ = DisableMouseCapture.write_ansi(&mut cmds);
        _ = DisableBracketedPaste.write_ansi(&mut cmds);
        _ = LeaveAlternateScreen.write_ansi(&mut cmds);
        _ = cursor::Show.write_ansi(&mut cmds);

        cmds
    }

    pub async fn draw<F, T>(&mut self, render: F) -> Result<T>
    where
        F: FnOnce(&mut Ui) -> T,
    {
        if !self.initialized {
            self.term.clear()?;

            self.stdout
                .send(Self::enter_sequence().into_bytes())
                .await?;

            self.initialized = true;
        }

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
                let cols = size.get(0).copied().unwrap_or(0);
                let rows = size.get(1).copied().unwrap_or(0);

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

    pub fn size(&self) -> UVec2 {
        self.size
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

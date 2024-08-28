use anyhow::{anyhow, Error, Result};
use futures_util::{Sink, SinkExt, Stream, StreamExt};
use ratatui::crossterm::event::{DisableBracketedPaste, EnableBracketedPaste};
use ratatui::crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::crossterm::{cursor, Command};
use ratatui::layout::Rect;
use ratatui::prelude::CrosstermBackend;
use ratatui::{Frame, Terminal, TerminalOptions, Viewport};
use std::io::{self, Write};
use std::mem;
use std::pin::Pin;
use termwiz::input::InputEvent;

pub struct Term {
    stdin: Pin<Box<dyn Stream<Item = Result<InputEvent>> + Send>>,
    stdout: Pin<Box<dyn Sink<Vec<u8>, Error = Error> + Send>>,
    term: Terminal<CrosstermBackend<WriterProxy>>,
    initialized: bool,
}

impl Term {
    // TODO buffer stdin if there's too many events at once
    pub fn new(
        stdin: impl Stream<Item = Result<InputEvent>> + Send + 'static,
        stdout: impl Sink<Vec<u8>, Error = Error> + Send + 'static,
        cols: u32,
        rows: u32,
    ) -> Result<Self> {
        let stdin = Box::pin(stdin);
        let stdout = Box::pin(stdout);

        let term = {
            let writer = WriterProxy::default();
            let backend = CrosstermBackend::new(writer);

            let opts = TerminalOptions {
                viewport: Viewport::Fixed(Self::viewport_rect(
                    cols as usize,
                    rows as usize,
                )),
            };

            Terminal::with_options(backend, opts)?
        };

        Ok(Self {
            stdin,
            stdout,
            term,
            initialized: false,
        })
    }

    pub async fn read(&mut self) -> Result<Option<InputEvent>> {
        let event = self
            .stdin
            .next()
            .await
            .ok_or_else(|| anyhow!("lost stdin"))??;

        if let InputEvent::Resized { cols, rows } = event {
            self.term.resize(Self::viewport_rect(cols, rows))?;

            Ok(None)
        } else {
            Ok(Some(event))
        }
    }

    pub async fn draw<F>(&mut self, render: F) -> Result<Rect>
    where
        F: FnOnce(&mut Frame),
    {
        if !self.initialized {
            self.term.clear()?;

            let mut cmds = String::new();

            _ = EnterAlternateScreen.write_ansi(&mut cmds);
            _ = EnableBracketedPaste.write_ansi(&mut cmds);

            self.stdout.send(cmds.into_bytes()).await?;
            self.initialized = true;
        }

        let area = self.term.draw(render)?.area;

        self.flush().await?;

        Ok(area)
    }

    pub fn exit_sequence() -> String {
        let mut cmds = String::new();

        _ = DisableBracketedPaste.write_ansi(&mut cmds);
        _ = LeaveAlternateScreen.write_ansi(&mut cmds);
        _ = cursor::Show.write_ansi(&mut cmds);

        cmds
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

    fn viewport_rect(cols: usize, rows: usize) -> Rect {
        Rect {
            x: 0,
            y: 0,
            width: cols.min(255) as u16,
            height: rows.min(255) as u16,
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

use crate::{Abort, Clear, Ui, UiLayout};
use anyhow::{anyhow, Error, Result};
use futures_util::{Sink, SinkExt, Stream, StreamExt};
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
use std::pin::Pin;
use std::sync::Arc;
use std::task::Waker;
use termwiz::escape::osc::Selection;
use termwiz::escape::OperatingSystemCommand;
use termwiz::input::{InputEvent, InputParser, MouseButtons, MouseEvent};
use tokio::select;
use tokio::sync::Notify;
use tracing::warn;

pub type Stdin = Pin<Box<dyn Stream<Item = Result<Vec<u8>>> + Send>>;
pub type Stdout = Pin<Box<dyn Sink<Vec<u8>, Error = Error> + Send>>;

pub struct Term {
    ty: TermType,
    stdin: Stdin,
    stdin_parser: InputParser,
    stdout: Stdout,
    term: Terminal<CrosstermBackend<WriterProxy>>,
    size: UVec2,
    mouse: TermMouse,
    event: Option<InputEvent>,
    notify: Arc<Notify>,
    waker: Waker,
}

impl Term {
    pub const CMD_RESIZE: u8 = 0x04;

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

        // ---

        term.clear()?;
        stdout.send(Self::enter_cmds().into_bytes()).await?;

        // ---

        let notify = Arc::new(Notify::new());

        let waker = waker_fn::waker_fn({
            let notify = notify.clone();

            move || {
                notify.notify_waiters();
            }
        });

        Ok(Self {
            ty,
            stdin,
            stdin_parser: Default::default(),
            stdout,
            term,
            size,
            mouse: Default::default(),
            event: Default::default(),
            notify,
            waker,
        })
    }

    pub fn enter_cmds() -> String {
        let mut cmds = String::new();

        _ = EnterAlternateScreen.write_ansi(&mut cmds);
        _ = EnableBracketedPaste.write_ansi(&mut cmds);
        _ = EnableMouseCapture.write_ansi(&mut cmds);
        _ = SetTitle("kartoffels").write_ansi(&mut cmds);

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

    pub async fn draw<F>(&mut self, render: F) -> Result<()>
    where
        F: FnOnce(&mut Ui),
    {
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
            let mut clipboard = Vec::new();

            self.term.draw(|frame| {
                let area = frame.area();

                render(&mut Ui {
                    ty: self.ty,
                    waker: &self.waker,
                    frame,
                    area,
                    mouse: self.mouse.report().as_ref(),
                    event: self.event.take().as_ref(),
                    clipboard: &mut clipboard,
                    layout: UiLayout::Col,
                    enabled: true,
                });
            })?;

            for payload in clipboard {
                self.copy_to_clipboard(payload).await?;
            }
        }

        self.flush().await?;

        Ok(())
    }

    pub async fn poll(&mut self) -> Result<()> {
        let bytes = select! {
            bytes = self.stdin.next() => Some(bytes),
            _ = self.notify.notified() => None,
        };

        if let Some(bytes) = bytes {
            let bytes = bytes.ok_or_else(|| anyhow!("lost stdin"))??;

            if let Some(size) = bytes.strip_prefix(&[Self::CMD_RESIZE]) {
                let cols = size.first().copied().unwrap_or(0);
                let rows = size.last().copied().unwrap_or(0);

                self.size = uvec2(cols as u32, rows as u32);
                self.term.resize(Self::viewport_rect(self.size))?;
            } else {
                let events = self.stdin_parser.parse_as_vec(&bytes, false);

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
        }

        Ok(())
    }

    pub async fn copy_to_clipboard(&mut self, payload: String) -> Result<()> {
        let cmd =
            OperatingSystemCommand::SetSelection(Selection::CLIPBOARD, payload);

        self.send(cmd.to_string().into_bytes()).await?;

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

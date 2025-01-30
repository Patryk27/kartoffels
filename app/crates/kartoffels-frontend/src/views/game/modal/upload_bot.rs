use super::{BotCount, BotPosition};
use crate::views::game::Event;
use anyhow::anyhow;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use kartoffels_store::{Session, SessionUploadInterest};
use kartoffels_ui::{
    theme, Button, FrameType, FromMarkdown, InputEvent, KeyCode, Modifiers,
    Spinner, Ui, UiWidget,
};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};
use std::cmp;
use std::time::Instant;

#[derive(Debug)]
pub struct UploadBotModal {
    request: UploadBotRequest<()>,
    interest: Option<SessionUploadInterest>,
    spinner: Spinner,
    alert: Option<Instant>,
}

impl UploadBotModal {
    pub fn new(request: UploadBotRequest<()>) -> Self {
        Self {
            request,
            interest: None,
            spinner: Default::default(),
            alert: Default::default(),
        }
    }

    pub fn render(&mut self, ui: &mut Ui<Event>, sess: &Session) {
        match ui.ty {
            FrameType::Ssh => {
                self.render_ssh(ui, sess);
            }

            FrameType::Web => {
                // For the web frame we don't render anything - rather, we
                // request the native file picker and wait until the bot is
                // uploaded via an HTTP endpoint.
                if self.interest.is_none() {
                    self.interest =
                        Some(sess.with(|sess| sess.request_upload()));
                }
            }
        }

        if let Some(upload) = &mut self.interest
            && let Some(src) = upload.try_recv()
        {
            ui.throw(Event::UploadBot {
                request: self.request.with_source(src),
            });
        }

        if let Some(event) = ui.event {
            self.handle(ui, event);
        }
    }

    fn render_ssh(&mut self, ui: &mut Ui<Event>, sess: &Session) {
        if ui.key(KeyCode::Char('v'), Modifiers::CTRL) {
            self.alert = Some(Instant::now());
        }

        // ---

        let body = Paragraph::new(vec![
            Line::md("run this:"),
            Line::md("    ./build --copy"),
            Line::md(""),
            Line::md(
                "... and then paste here (`Ctrl+Shift+V`, `Cmd+V` etc.) to \
                 upload the bot",
            ),
            Line::md(""),
            Line::md(
                "alternatively, if your terminal doesn't support bracketed \
                 paste, use:",
            ),
            Line::md(&format!("    ./build --upload {}", sess.id())),
        ])
        .wrap(Wrap::default());

        let width = cmp::min(ui.area.width - 10, 70);
        let body_height = body.line_count(width) as u16;
        let height = body_height + 4;

        // ---

        ui.info_window(width, height, Some(" upload-bot "), |ui| {
            ui.widget(&body);
            ui.space(body_height + 1);

            if let Some(alert) = &self.alert {
                ui.line(
                    Line::raw("try using Ctrl+Shift+V instead of Ctrl+V")
                        .fg(theme::YELLOW)
                        .bold()
                        .centered(),
                );

                ui.space(1);

                if alert.elapsed().as_secs() >= 3 {
                    self.alert = None;
                }
            } else {
                let spinner = self.spinner.as_span();

                Line::from_iter([
                    spinner.clone(),
                    Span::raw(" waiting "),
                    spinner,
                ])
                .centered()
                .render(ui);

                ui.space(2);
            }

            ui.row(|ui| {
                Button::new(KeyCode::Escape, "cancel")
                    .throwing(Event::CloseModal)
                    .render(ui);

                if Button::new(KeyCode::Char('c'), "copy-session-id")
                    .right_aligned()
                    .render(ui)
                    .pressed
                {
                    ui.throw(Event::Copy {
                        payload: sess.id().to_string(),
                    });
                }
            });
        });
    }

    fn handle(&mut self, ui: &mut Ui<Event>, event: &InputEvent) {
        if let InputEvent::Paste(src) = event {
            self.handle_paste(ui, src);
        }
    }

    fn handle_paste(&mut self, ui: &mut Ui<Event>, src: &str) {
        if src.is_empty() {
            // A bit hacky, but that's the most sane way for the web terminal to
            // inform us that user has cancelled the file picker
            ui.throw(Event::CloseModal);
        } else {
            let src = src.trim().replace('\r', "");
            let src = src.trim().replace('\n', "");

            let src = match BASE64_STANDARD.decode(src) {
                Ok(src) => src,

                Err(err) => {
                    ui.throw(Event::OpenErrorModal {
                        error: anyhow!("{err}")
                            .context("couldn't decode pasted content")
                            .context("couldn't upload bot"),
                    });

                    return;
                }
            };

            ui.throw(Event::UploadBot {
                request: self.request.with_source(src),
            });
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct UploadBotRequest<S> {
    pub source: S,
    pub position: BotPosition,
    pub count: BotCount,
}

impl<S> UploadBotRequest<S> {
    pub fn new(source: S) -> Self {
        Self {
            source,
            position: Default::default(),
            count: Default::default(),
        }
    }

    pub fn with_source<S2>(&self, source: S2) -> UploadBotRequest<S2> {
        UploadBotRequest {
            source,
            position: self.position,
            count: self.count,
        }
    }
}

use super::{BotCount, BotPosition};
use crate::views::game::{BotSource, Event};
use kartoffels_store::{SessionId, SessionUploadInterest, Store};
use kartoffels_ui::{
    theme, Button, FromMarkdown, InputEvent, KeyCode, Modifiers, Spinner,
    TermFrontend, Ui, UiWidget,
};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, WidgetRef, Wrap};
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
    pub fn new(
        request: UploadBotRequest<()>,
        store: &Store,
        sess: SessionId,
    ) -> Self {
        let interest = store.with_session(sess, |sess| sess.request_upload());

        Self {
            request,
            interest,
            spinner: Default::default(),
            alert: Default::default(),
        }
    }

    pub fn render(&mut self, ui: &mut Ui<Event>, sess: SessionId) {
        match ui.frontend {
            TermFrontend::Ssh => {
                self.render_ssh(ui, sess);
            }

            TermFrontend::Web => {
                // We don't render anything for the web terminal, since it opens
                // a file picker instead
            }
        }

        if let Some(InputEvent::Paste(src)) = ui.event {
            if src.is_empty() {
                // A bit hacky, but that's the most sane way for the web
                // terminal to inform us that user has cancelled the file picker
                ui.throw(Event::CloseModal);
            } else {
                ui.throw(Event::UploadBot {
                    request: self
                        .request
                        .with_source(BotSource::Base64(src.to_owned())),
                });
            }
        }

        if let Some(upload) = &mut self.interest {
            if let Some(src) = upload.try_recv() {
                ui.throw(Event::UploadBot {
                    request: self.request.with_source(BotSource::Binary(src)),
                });
            }
        }
    }

    fn render_ssh(&mut self, ui: &mut Ui<Event>, sess: SessionId) {
        let spinner = self.spinner.as_span();

        if ui.key(KeyCode::Char('v'), Modifiers::CTRL) {
            self.alert = Some(Instant::now());
        }

        // ---

        let width = cmp::min(ui.area.width - 10, 70);

        let body = Self::body(sess);
        let body_height = body.line_count(width) as u16;

        let height = body_height + 4;

        ui.info_window(width, height, Some(" upload-bot "), |ui| {
            body.render_ref(ui.area, ui.buf);
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
                    ui.copy(sess.to_string());
                }
            });
        });
    }

    fn body(sess: SessionId) -> Paragraph<'static> {
        Paragraph::new(vec![
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
            Line::md(&format!("    ./build --upload {sess}")),
        ])
        .wrap(Wrap::default())
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct UploadBotRequest<S> {
    pub source: S,
    pub position: BotPosition,
    pub count: BotCount,
}

impl<S> UploadBotRequest<S> {
    pub fn with_source<S2>(&self, source: S2) -> UploadBotRequest<S2> {
        UploadBotRequest {
            source,
            position: self.position,
            count: self.count,
        }
    }
}

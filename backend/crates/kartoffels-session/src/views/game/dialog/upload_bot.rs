use crate::views::game::Event;
use itertools::Either;
use kartoffels_store::{SessionId, SessionUploadInterest, Store};
use kartoffels_ui::{theme, Button, FromMarkdown, Render, Spinner, Ui};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, WidgetRef, Wrap};
use std::cmp;
use std::time::Instant;
use termwiz::input::{InputEvent, KeyCode, Modifiers};

#[derive(Debug)]
pub struct UploadBotDialog {
    spinner: Spinner,
    ctrlv_alert: Option<Instant>,
    upload: Option<SessionUploadInterest>,
}

impl UploadBotDialog {
    pub fn new(store: &Store, sess: SessionId) -> Self {
        let upload = store.with_session(sess, |sess| sess.request_upload());

        Self {
            spinner: Default::default(),
            ctrlv_alert: Default::default(),
            upload,
        }
    }

    pub fn render(&mut self, ui: &mut Ui<Event>, sess: SessionId) {
        if ui.ty.is_ssh() {
            self.render_ssh(ui, sess);
        }

        if let Some(InputEvent::Paste(src)) = ui.event {
            if src.is_empty() {
                // A bit hacky, but that's the most sane way for the http
                // frontend to inform us that user has cancelled the uploading
                ui.throw(Event::CloseDialog);
            } else {
                ui.throw(Event::CreateBot {
                    src: Either::Left(src.to_owned()),
                    pos: None,
                    follow: true,
                });
            }
        }

        if let Some(upload) = &mut self.upload {
            if let Some(src) = upload.try_recv() {
                ui.throw(Event::CreateBot {
                    src: Either::Right(src),
                    pos: None,
                    follow: true,
                });
            }
        }
    }

    fn render_ssh(&mut self, ui: &mut Ui<Event>, sess: SessionId) {
        let spinner = self.spinner.as_span();

        if ui.key(KeyCode::Char('v'), Modifiers::CTRL) {
            self.ctrlv_alert = Some(Instant::now());
        }

        // ---

        let text = Self::build_ssh_text(sess);
        let width = cmp::min(ui.area.width - 10, 60);
        let text_height = text.line_count(width) as u16;

        let height = if self.ctrlv_alert.is_some() {
            text_height + 6
        } else {
            text_height + 4
        };

        ui.info_window(width, height, Some(" upload-bot "), |ui| {
            text.render_ref(ui.area, ui.buf);
            ui.space(text_height + 1);

            Line::from_iter([spinner.clone(), Span::raw(" waiting "), spinner])
                .centered()
                .render(ui);

            ui.space(2);

            if let Some(alert) = &mut self.ctrlv_alert {
                ui.line(
                    Line::raw("try using Ctrl+Shift+V instead of Ctrl+V")
                        .fg(theme::YELLOW)
                        .bold()
                        .centered(),
                );

                ui.space(1);

                if alert.elapsed().as_secs() >= 3 {
                    self.ctrlv_alert = None;
                }
            }

            ui.row(|ui| {
                Button::new(KeyCode::Escape, "cancel")
                    .throwing(Event::CloseDialog)
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

    fn build_ssh_text(sess: SessionId) -> Paragraph<'static> {
        Paragraph::new(vec![
            Line::md("run:"),
            Line::md(""),
            Line::md("    ./build --copy"),
            Line::md("  or").fg(theme::GRAY),
            Line::md("    ./build.bat --copy"),
            Line::md(""),
            Line::md(
                "... and then paste your clipboard here ([`Ctrl+Shift+V`], \
                 [`Cmd+V`] etc.) to upload the bot",
            ),
            Line::md(""),
            Line::md(
                "alternatively, if your terminal doesn't support bracketed \
                 paste, use:",
            ),
            Line::md(""),
            Line::md(&format!("    ./build --upload {sess}")),
        ])
        .wrap(Wrap::default())
    }
}

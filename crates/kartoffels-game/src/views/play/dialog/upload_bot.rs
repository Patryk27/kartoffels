use super::DialogResponse;
use kartoffels_ui::{theme, Button, Spinner, Ui};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget, WidgetRef, Wrap};
use std::cmp;
use std::pin::Pin;
use std::sync::LazyLock;
use std::time::Duration;
use termwiz::input::{InputEvent, KeyCode, Modifiers};
use tokio::time;

static TEXT: LazyLock<Paragraph<'static>> = LazyLock::new(|| {
    Paragraph::new(vec![
        Line::raw("if you're following the standard template, run:"),
        Line::raw(""),
        Line::raw("    ./build --copy").fg(theme::WASHED_PINK),
        Line::raw("  or").fg(theme::GRAY),
        Line::raw("    ./build.bat --copy").fg(theme::WASHED_PINK),
        Line::raw(""),
        Line::raw("... and then paste your clipboard here (Ctrl+Shift+V, Cmd+V etc.) to upload the bot"),
        Line::raw(""),
        Line::raw("if you're not following the template, you have to build the *.elf file, base64-encode it and then paste it here, like:"),
        Line::raw(""),
        Line::raw("    cargo build --release").fg(theme::WASHED_PINK),
        Line::raw("    base64 target/foo/bar | wl-copy").fg(theme::WASHED_PINK),
    ])
    .wrap(Wrap::default())
});

#[derive(Debug, Default)]
pub struct UploadBotDialog {
    spinner: Spinner,
    ctrlv_alert: Option<Pin<Box<time::Sleep>>>,
}

impl UploadBotDialog {
    pub fn render(&mut self, ui: &mut Ui) -> Option<DialogResponse> {
        let mut resp = None;

        if ui.ty().is_ssh() {
            let width = cmp::min(ui.area().width - 10, 60);
            let text_height = TEXT.line_count(width) as u16;

            let spinner = self.spinner.as_span(ui);

            if ui.key(KeyCode::Char('v'), Modifiers::CTRL) {
                self.ctrlv_alert =
                    Some(Box::pin(time::sleep(Duration::from_secs(3))));
            }

            let height = if self.ctrlv_alert.is_some() {
                text_height + 6
            } else {
                text_height + 4
            };

            ui.info_window(width, height, Some(" uploading a bot "), |ui| {
                TEXT.render_ref(ui.area(), ui.buf());

                ui.space(text_height + 1);

                Line::from_iter([
                    spinner.clone(),
                    Span::raw(" waiting "),
                    spinner,
                ])
                .centered()
                .render(ui.area(), ui.buf());

                ui.space(2);

                if let Some(alert) = &mut self.ctrlv_alert {
                    ui.line(
                        Line::raw("try using Ctrl+Shift+V instead of Ctrl+V")
                            .fg(theme::POTATO)
                            .bold()
                            .centered(),
                    );

                    ui.space(1);

                    if ui.poll(alert).is_ready() {
                        self.ctrlv_alert = None;
                    }
                }

                if Button::new(KeyCode::Escape, "cancel").render(ui).pressed {
                    resp = Some(DialogResponse::Close);
                }
            });
        }

        if let Some(InputEvent::Paste(src)) = ui.event() {
            if src.is_empty() {
                // A bit hacky, but that's the most sane way for the http
                // frontend to inform us that user has cancelled the uploading
                resp = Some(DialogResponse::Close);
            } else {
                resp = Some(DialogResponse::UploadBot(src.to_owned()));
            }
        }

        resp
    }
}

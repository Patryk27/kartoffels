use super::DialogResponse;
use crate::{theme, Button, Spinner, Ui};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget, WidgetRef, Wrap};
use std::cmp;
use std::sync::LazyLock;
use termwiz::input::{InputEvent, KeyCode};

static TEXT: LazyLock<Paragraph<'static>> = LazyLock::new(|| {
    Paragraph::new(vec![
        Line::raw("if you're following the standard template, run:"),
        Line::raw(""),
        Line::raw("    ./build --copy").fg(theme::WASHED_PINK),
        Line::raw("  or").fg(theme::GRAY),
        Line::raw("    ./build.bat --copy").fg(theme::WASHED_PINK),
        Line::raw(""),
        Line::raw(
            "... and then paste your clipboard here (Ctrl+Shift+V, Cmd+V \
             etc.) to upload the bot",
        ),
        Line::raw(""),
        Line::raw(
            "if you're not following the template, you have to just build \
             the *.elf file, base64-encode it and then paste it here",
        ),
    ])
    .wrap(Wrap::default())
});

#[derive(Debug, Default)]
pub struct UploadBotDialog {
    spinner: Spinner,
}

impl UploadBotDialog {
    pub fn render(&mut self, ui: &mut Ui) -> Option<DialogResponse> {
        let mut resp = None;

        if ui.ty().is_ssh() {
            let width = cmp::min(ui.area().width - 10, 60);
            let lines = TEXT.line_count(width) as u16;
            let height = lines + 4;

            let spinner = self.spinner.as_span(ui);

            ui.info_dialog(width, height, Some(" uploading a bot "), |ui| {
                TEXT.render_ref(ui.area(), ui.buf());

                ui.space(lines + 1);

                Line::from_iter([
                    spinner.clone(),
                    Span::raw(" waiting "),
                    spinner,
                ])
                .centered()
                .render(ui.area(), ui.buf());

                ui.space(2);

                if Button::new(KeyCode::Escape, "cancel").render(ui).pressed {
                    resp = Some(DialogResponse::Close);
                }
            });
        }

        if let Some(InputEvent::Paste(src)) = ui.event() {
            if src.is_empty() {
                // A bit hacky, but that's the most sane way for the http
                // frontend to inform us that user's cancelled the uploading
                resp = Some(DialogResponse::Close);
            } else {
                resp = Some(DialogResponse::UploadBot(src.to_owned()));
            }
        }

        resp
    }
}

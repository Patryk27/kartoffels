use super::DialogEvent;
use crate::{theme, Button, RectExt, Ui};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget, Wrap};
use termwiz::input::KeyCode;
use tokio::time::{self, Interval};

#[derive(Debug)]
pub struct UploadBotDialog {
    pub spinner_interval: Interval,
    pub spinner_icon: usize,
}

impl UploadBotDialog {
    const SPINNER: &[&str] = &["|", "/", "-", "\\"];

    pub fn render(&self, ui: &mut Ui) -> Option<DialogEvent> {
        let spinner = Self::SPINNER[self.spinner_icon % Self::SPINNER.len()];
        let spinner = Span::raw(spinner).fg(theme::GREEN);

        let para = Paragraph::new(vec![
            Line::raw("if you're following the standard template, run:"),
            Line::raw(""),
            Line::raw("    ./build --copy").fg(theme::WASHED_PINK),
            Line::raw("  (or)").fg(theme::GRAY),
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
            Line::raw(""),
            Line::from_iter([spinner.clone(), Span::raw(" waiting "), spinner])
                .centered(),
        ])
        .wrap(Wrap::default());

        let width = 60;
        let height = para.line_count(width) as u16 + 2;

        let mut event = None;

        ui.info_dialog(width, height, Some(" uploading a bot "), |ui| {
            para.render(ui.area(), ui.buf());

            ui.clamp(ui.area().footer(), |ui| {
                if Button::new(KeyCode::Escape, "cancel", true)
                    .render(ui)
                    .activated
                {
                    event = Some(DialogEvent::Close);
                }
            });
        });

        event
    }
}

impl Default for UploadBotDialog {
    fn default() -> Self {
        Self {
            spinner_interval: time::interval(theme::SPINNER_INTERVAL),
            spinner_icon: 0,
        }
    }
}

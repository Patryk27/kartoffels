use crate::views::game::Event;
use kartoffels_ui::{theme, Button, FromMarkdown, Render, Spinner, Ui};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, WidgetRef, Wrap};
use std::cmp;
use std::sync::LazyLock;
use std::time::Instant;
use termwiz::input::{InputEvent, KeyCode, Modifiers};

static TEXT: LazyLock<Paragraph<'static>> = LazyLock::new(|| {
    Paragraph::new(vec![
        Line::md("if you're following the standard template, run:"),
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
            "if you're not following the standard template, you better consult \
             kartoffels wiki on github, you hacker",
        ),
    ])
    .wrap(Wrap::default())
});

#[derive(Debug, Default)]
pub struct UploadBotDialog {
    spinner: Spinner,
    ctrlv_alert: Option<Instant>,
}

impl UploadBotDialog {
    pub fn render(&mut self, ui: &mut Ui<Event>) {
        if ui.ty().is_ssh() {
            let width = cmp::min(ui.area().width - 10, 60);
            let text_height = TEXT.line_count(width) as u16;

            let spinner = self.spinner.as_span();

            if ui.key(KeyCode::Char('v'), Modifiers::CTRL) {
                self.ctrlv_alert = Some(Instant::now());
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

                Button::new(KeyCode::Escape, "cancel")
                    .throwing(Event::CloseDialog)
                    .render(ui);
            });
        }

        if let Some(InputEvent::Paste(src)) = ui.event() {
            if src.is_empty() {
                // A bit hacky, but that's the most sane way for the http
                // frontend to inform us that user has cancelled the uploading
                ui.throw(Event::CloseDialog);
            } else {
                ui.throw(Event::UploadBot(src.to_owned()));
            }
        }
    }
}

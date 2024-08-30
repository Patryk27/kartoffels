use super::DialogEvent;
use crate::{theme, Action, BlockExt, IntervalExt, LayoutExt, RectExt};
use ratatui::buffer::Buffer;
use ratatui::layout::{Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Widget, Wrap};
use termwiz::input::InputEvent;
use tokio::time::{self, Interval};

#[derive(Debug)]
pub struct UploadBotDialog {
    pub spinner_interval: Interval,
    pub spinner_icon: usize,
}

impl UploadBotDialog {
    const SPINNER: &[&str] = &["|", "/", "-", "\\"];

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
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
        let height = para.line_count(width) as u16;

        let area = Block::dialog_info(
            Some(" uploading a bot "),
            Layout::dialog(width, height + 2, area),
            buf,
        );

        para.render(area, buf);

        Line::from(Action::new("esc", "cancel", true))
            .left_aligned()
            .render(area.footer(), buf);
    }

    pub fn handle(&mut self, event: InputEvent) -> Option<DialogEvent> {
        if let InputEvent::Paste(src) = event {
            return Some(DialogEvent::UploadBot(src));
        }

        None
    }

    pub async fn tick(&mut self) {
        self.spinner_interval.tick().await;
        self.spinner_icon += 1;
    }
}

impl Default for UploadBotDialog {
    fn default() -> Self {
        Self {
            spinner_interval: time::interval(theme::SPINNER_INTERVAL)
                .skipping_first(),
            spinner_icon: 0,
        }
    }
}

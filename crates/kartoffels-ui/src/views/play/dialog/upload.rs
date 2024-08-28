use crate::{theme, Action, BlockExt, Clear};
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Layout, Offset, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph, Widget};
use termwiz::input::InputEvent;

#[derive(Debug, Default)]
pub struct UploadDialog;

impl UploadDialog {
    pub fn handle(&mut self, event: InputEvent) -> UploadDialogOutcome {
        if let InputEvent::Paste(src) = event {
            return UploadDialogOutcome::Ready(src);
        }

        UploadDialogOutcome::None
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        let [_, area, _] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(62),
            Constraint::Fill(1),
        ])
        .areas(area);

        let [_, area, _] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(14),
            Constraint::Fill(2),
        ])
        .areas(area);

        let area = Block::bordered()
            .border_style(Style::new().fg(theme::GREEN).bg(theme::BG))
            .title("uploading a bot")
            .title_alignment(Alignment::Center)
            .render_and_measure(area, buf);

        Clear.render(area, buf);

        Paragraph::new(vec![
            Line::raw("if you're following the standard template, run:"),
            Line::raw(""),
            Line::raw("    ./build --copy").fg(theme::WASHED_PINK),
            Line::raw("  (or)").fg(theme::GRAY),
            Line::raw("    ./build.bat --copy").fg(theme::WASHED_PINK),
            Line::raw(""),
            Line::raw(
                "... and then paste your clipboard here to upload the bot",
            ),
            Line::raw(""),
            Line::raw(
                "if you're not following the template, you have to just build",
            ),
            Line::raw(
                "the *.elf file, base64-encode it and then paste it here",
            ),
        ])
        .render(area, buf);

        let area = area.offset(Offset { x: 0, y: 11 });

        Line::from(Action::new("esc", "cancel", true))
            .right_aligned()
            .render(area, buf);
    }
}

#[derive(Debug)]
pub enum UploadDialogOutcome {
    Ready(String),
    None,
}

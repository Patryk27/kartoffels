use crate::{theme, FromMarkdown, Ui};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Padding};
use std::sync::LazyLock;

#[rustfmt::skip]
static TEXT: LazyLock<Text<'static>> = LazyLock::new(|| {
    Text::from_iter([
        Line::md("welcome to kartoffels, a game where you're given a potato:"),
        Line::md(""),
        Line::md("     ██████     ").fg(theme::YELLOW),
        Line::md("   ██░░░░░░██   ").fg(theme::YELLOW),
        Line::md(" ██░░░░░░░░░░██ ").fg(theme::YELLOW),
        Line::md(" ██░░░░░░░░░░██ ").fg(theme::YELLOW),
        Line::md("   ██░░░░░░░░██ ").fg(theme::YELLOW),
        Line::md("   oo████████oo ").fg(theme::YELLOW),
        Line::md("   oo        oo ").fg(theme::YELLOW),
        Line::md(""),
        Line::md("... and your job is to implement a firmware for it!"),
        Line::md(""),
        Line::md("armed with *64 khz cpu* and *128 kib ram*, you can compete"),
        Line::md("against other players, solve single-player challenges"),
        Line::md("or simply make up your own goal - have fun!"),
    ])
    .centered()
});

#[derive(Debug)]
pub struct Header;

impl Header {
    pub fn width() -> u16 {
        58 + 2 + 2
    }

    pub fn height() -> u16 {
        TEXT.lines.len() as u16 + 2
    }

    pub fn render<T>(ui: &mut Ui<T>) {
        let block = Block::bordered()
            .border_style(Style::new().fg(theme::GREEN).bg(theme::BG))
            .padding(Padding::horizontal(1));

        ui.block(block, |ui| {
            ui.add(&*TEXT);
        });
    }
}

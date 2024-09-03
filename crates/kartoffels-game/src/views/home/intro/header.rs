use kartoffels_ui::{theme, Ui};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Padding, Widget, WidgetRef};
use std::sync::LazyLock;

static TEXT: LazyLock<Text<'static>> = LazyLock::new(|| {
    Text::from_iter([
        Line::raw("welcome to kartoffels, a game where you're given a potato:"),
        Line::raw(""),
        Line::raw("     ██████     ").fg(theme::POTATO),
        Line::raw("   ██░░░░░░██   ").fg(theme::POTATO),
        Line::raw(" ██░░░░░░░░░░██ ").fg(theme::POTATO),
        Line::raw(" ██░░░░░░░░░░██ ").fg(theme::POTATO),
        Line::raw("   ██░░░░░░░░██ ").fg(theme::POTATO),
        Line::raw("   oo████████oo ").fg(theme::POTATO),
        Line::raw("   oo        oo ").fg(theme::POTATO),
        Line::raw(""),
        Line::raw("... and your job is to implement a firmware for it"),
        Line::raw(""),
        Line::from_iter([
            Span::raw("you've got "),
            Span::raw("64 khz cpu ").bold(),
            Span::raw("and "),
            Span::raw("128 kb of ram ").bold(),
            Span::raw("at hand, and you"),
        ]),
        Line::raw("can either compete against other players in the online"),
        Line::raw("play or indulge yourself with single-player challenges"),
        Line::raw(""),
        Line::raw("have fun!"),
        Line::raw("~pwy"),
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

    pub fn render(ui: &mut Ui) {
        let block = Block::bordered()
            .border_style(Style::new().fg(theme::GREEN).bg(theme::BG))
            .padding(Padding::horizontal(1));

        let area = {
            let inner_area = block.inner(ui.area());

            block.render(ui.area(), ui.buf());
            inner_area
        };

        TEXT.render_ref(area, ui.buf())
    }
}

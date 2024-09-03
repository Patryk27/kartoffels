use super::DialogResponse;
use kartoffels_ui::{theme, Button, RectExt, Ui};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, WidgetRef, Wrap};
use std::sync::LazyLock;
use termwiz::input::KeyCode;

static TEXT: LazyLock<Paragraph<'static>> = LazyLock::new(|| {
    Paragraph::new(vec![
        Line::raw("welcome to kartoffels 🫡"),
        Line::raw(""),
        Line::from_iter([
            Span::raw("there's a built-in tutorial which you can open by pressing ["),
            Span::raw("t").fg(theme::GREEN),
            Span::raw("] now, but if you're more into discovering things yourself, here's a couple of tips to start you off:"),
        ]),
        Line::raw(""),
        Line::from_iter([
            Span::raw("- run `"),
            Span::raw("git clone https://github.com/Patryk27/kartoffel").fg(theme::WASHED_PINK),
            Span::raw("` to download the template repository"),
        ]),
        Line::raw("- once you have the template, see README.md for building instructions"),
        Line::from_iter([
            Span::raw("- use your ide's `"),
            Span::raw("go to definition").fg(theme::WASHED_PINK),
            Span::raw("` feature to discover how robot-specific functions work"),
        ]),
        Line::raw("- press [u] to upload your bot"),
        Line::raw("- press [w/a/s/d] to navigate the map"),
        Line::raw("- you can also use the mouse!"),
        Line::raw("- bots are represented with the `@` char, `.` is floor etc."),
        Line::raw("- you're smart, you'll figure out the rest"),
    ])
    .wrap(Wrap::default())
});

#[derive(Debug, Default)]
pub struct HelpDialog;

impl HelpDialog {
    pub fn render(&self, ui: &mut Ui) -> Option<DialogResponse> {
        let mut resp = None;

        let width = ui.area().width - 10;
        let height = TEXT.line_count(width) as u16 + 2;

        ui.info_dialog(width, height, Some(" help "), |ui| {
            TEXT.render_ref(ui.area(), ui.buf());

            ui.clamp(ui.area().footer(1), |ui| {
                if Button::new(KeyCode::Escape, "close").render(ui).pressed {
                    resp = Some(DialogResponse::Close);
                }

                if Button::new(KeyCode::Char('t'), "open tutorial")
                    .right_aligned()
                    .render(ui)
                    .pressed
                {
                    resp = Some(DialogResponse::OpenTutorial);
                }
            })
        });

        resp
    }
}

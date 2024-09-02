use super::DialogResponse;
use crate::{Button, RectExt, Ui};
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, WidgetRef, Wrap};
use std::cmp;
use std::sync::LazyLock;
use termwiz::input::KeyCode;

static TEXT: LazyLock<Paragraph<'static>> = LazyLock::new(|| {
    Paragraph::new(vec![
        Line::raw("hey there soldier and welcome to kartoffels 🫡🫡🫡"),
        Line::raw(""),
        Line::raw(
            "the game has a built-in tutorial which you can open by pressing \
             [t] now, but if you're more into discovering things yourself, \
             here's a couple of tips:",
        ),
        Line::raw(""),
        Line::raw(
            "- run `git clone https://github.com/Patryk27/kartoffel` to get a \
             starting point - see README.md there for building instructions",
        ),
        Line::raw(
            "- use your ide's `go to definition` feature to discover how \
             robot-specific functions, such as `radar_scan()`, work",
        ),
        Line::raw("- press [u] to upload your bot"),
        Line::raw("- press [w/a/s/d] to navigate the map"),
        Line::raw(
            "- bots are represented with the `@` char, `.` is floor etc.",
        ),
    ])
    .wrap(Wrap::default())
});

#[derive(Debug, Default)]
pub struct HelpDialog;

impl HelpDialog {
    pub fn render(&self, ui: &mut Ui) -> Option<DialogResponse> {
        let mut resp = None;

        let width = cmp::min(ui.area().width - 10, 60);
        let height = TEXT.line_count(width) as u16 + 2;

        ui.info_dialog(width, height, Some(" help "), |ui| {
            TEXT.render_ref(ui.area(), ui.buf());

            ui.clamp(ui.area().footer(1), |ui| {
                if Button::new(KeyCode::Escape, "close").render(ui).pressed {
                    resp = Some(DialogResponse::Close);
                }

                if Button::new(KeyCode::Char('t'), "go to tutorial")
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

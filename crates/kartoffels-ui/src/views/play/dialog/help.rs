use super::DialogEvent;
use crate::{Button, RectExt, Ui};
use indoc::indoc;
use ratatui::widgets::{Paragraph, Widget, Wrap};
use std::cmp;
use termwiz::input::KeyCode;

#[derive(Debug, Default)]
pub struct HelpDialog;

impl HelpDialog {
    const TEXT: &str = indoc! {"
        hey there soldier and welcome to kartoffels 🫡🫡🫡

        the game has a built-in tutorial which you can open by pressing [t] \
        now, but if you're more into discovering things yourself, here's a \
        couple of tips:

        - run `git clone https://github.com/Patryk27/kartoffel` to get a \
        starting point - see README.md there for building instructions
        - use your ide's `go to definition` feature to discover how \
        robot-specific functions, such as `radar_scan()`, work
        - press [u] to upload your bot
        - press [w/a/s/d] to navigate the map
        - bots are represented with the `@` char, `.` is floor etc.
    "};

    pub fn render(&self, ui: &mut Ui) -> Option<DialogEvent> {
        let text = Paragraph::new(Self::TEXT).wrap(Wrap::default());
        let width = cmp::min(ui.area().width - 10, 60);
        let height = text.line_count(width) as u16 + 2;

        let mut event = None;

        ui.info_dialog(width, height, Some(" help "), |ui| {
            text.render(ui.area(), ui.buf());

            ui.clamp(ui.area().footer(), |ui| {
                if Button::new(KeyCode::Escape, "go back").render(ui).activated
                {
                    event = Some(DialogEvent::Close);
                }

                if Button::new(KeyCode::Char('t'), "go to tutorial")
                    .right()
                    .render(ui)
                    .activated
                {
                    event = Some(DialogEvent::OpenTutorial);
                }
            })
        });

        event
    }
}

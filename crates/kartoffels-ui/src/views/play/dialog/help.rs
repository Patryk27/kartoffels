use super::DialogEvent;
use crate::{Action, BlockExt, LayoutExt, RectExt};
use indoc::indoc;
use ratatui::layout::Layout;
use ratatui::prelude::{Buffer, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph, Widget, Wrap};
use std::cmp;
use termwiz::input::{InputEvent, KeyCode, Modifiers};

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

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        let text = Paragraph::new(Self::TEXT).wrap(Wrap::default());

        let width = cmp::min(area.width - 10, 60);
        let height = text.line_count(width) as u16;

        let area = Block::dialog_info(
            Some(" help "),
            Layout::dialog(width, height + 2, area),
            buf,
        );

        text.render(area, buf);

        Line::from(Action::new("esc", "go back", true))
            .left_aligned()
            .render(area.footer(), buf);

        Line::from(Action::new("t", "go to tutorial", true))
            .right_aligned()
            .render(area.footer(), buf);
    }

    pub fn handle(&mut self, event: InputEvent) -> Option<DialogEvent> {
        if let InputEvent::Key(event) = event {
            match (event.key, event.modifiers) {
                (KeyCode::Escape, _) => {
                    return Some(DialogEvent::Close);
                }

                (KeyCode::Char('t'), Modifiers::NONE) => {
                    return Some(DialogEvent::OpenTutorial);
                }

                _ => (),
            }
        }

        None
    }
}

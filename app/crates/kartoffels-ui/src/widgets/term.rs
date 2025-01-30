use super::Input;
use crate::{Ui, UiWidget};
use ratatui::layout::{Constraint, Layout};
use ratatui::widgets::Paragraph;
use std::fmt;
use termwiz::input::{KeyCode, Modifiers};

#[derive(Clone, Debug, Default)]
pub struct Term {
    stdin: Input,
    stdout: String,
    prev_stdin: Option<String>,
}

impl Term {
    pub const MAX_LENGTH: usize = 32 * 1024;
}

impl fmt::Write for Term {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.stdout.write_str(s)?;

        if self.stdout.len() > Self::MAX_LENGTH {
            let split_at = self.stdout.len() - Self::MAX_LENGTH;

            for offset in 0..4 {
                let split_at = split_at + offset;

                if self.stdout.is_char_boundary(split_at) {
                    self.stdout = self.stdout.split_off(split_at);
                    break;
                }
            }
        }

        Ok(())
    }
}

impl UiWidget<String> for &mut Term {
    fn render(self, ui: &mut Ui<String>) -> Self::Response {
        let [stdout_area, _, stdin_area] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas(ui.area);

        let stdout = {
            let stdout =
                Paragraph::new(self.stdout.as_str()).wrap(Default::default());

            let scroll = stdout
                .line_count(stdout_area.width)
                .saturating_sub(stdout_area.height as usize);

            stdout.scroll((scroll as u16, 0))
        };

        ui.add_at(stdout_area, stdout);
        ui.add_at(stdin_area, &mut self.stdin);

        // ---

        if ui.key(KeyCode::UpArrow, Modifiers::NONE)
            && let Some(prev_stdin) = &self.prev_stdin
        {
            *self.stdin.value_mut() = prev_stdin.clone();
        }

        if ui.key(KeyCode::Enter, Modifiers::NONE) {
            let stdin = self.stdin.take_value().trim().to_owned();

            if !stdin.is_empty() {
                self.prev_stdin = Some(stdin.clone());

                ui.throw(stdin);
            }
        }
    }
}

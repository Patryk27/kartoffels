use crate::{BgMap, Button, Frame};
use anyhow::{Error, Result};
use kartoffels_utils::ErrorExt;
use ratatui::layout::{Constraint, Layout};
use ratatui::widgets::Paragraph;
use termwiz::input::KeyCode;
use tracing::debug;

pub async fn run(frame: &mut Frame, bg: &BgMap, err: Error) -> Result<()> {
    debug!(?err, "run()");

    let err = Paragraph::new(err.to_fmt_string()).wrap(Default::default());

    loop {
        let go_back = frame
            .tick(|ui| {
                let width = 60;
                let height = err.line_count(width) as u16 + 2;

                ui.add(bg);

                ui.error_window(width, height, Some(" ouch "), |ui| {
                    let [text_area, _, footer_area] = Layout::vertical([
                        Constraint::Fill(1),
                        Constraint::Length(1),
                        Constraint::Length(1),
                    ])
                    .areas(ui.area);

                    ui.add_at(text_area, &err);

                    ui.add_at(footer_area, {
                        Button::new("close", KeyCode::Enter)
                            .throwing(())
                            .right_aligned()
                    });
                });
            })
            .await?
            .is_some();

        if go_back {
            return Ok(());
        }
    }
}

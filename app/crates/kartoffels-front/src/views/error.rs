use crate::{BgMap, Frame};
use anyhow::{Error, Result};
use kartoffels_utils::ErrorExt;
use ratatui::layout::{Constraint, Layout};
use ratatui::widgets::Paragraph;
use termwiz::input::KeyCode;
use tracing::info;

pub async fn run(frame: &mut Frame, bg: &BgMap, err: Error) -> Result<()> {
    info!(?err, "run()");

    let text = Paragraph::new(err.to_fmt_string()).wrap(Default::default());

    loop {
        let go_back = frame
            .render(|ui| {
                let width = 60;
                let height = text.line_count(width) as u16 + 2;

                ui.add(bg);

                ui.emodal(width, height, Some("ouch"), |ui| {
                    let [text_area, _, footer_area] = Layout::vertical([
                        Constraint::Fill(1),
                        Constraint::Length(1),
                        Constraint::Length(1),
                    ])
                    .areas(ui.area);

                    ui.add_at(text_area, &text);

                    ui.at(footer_area, |ui| {
                        ui.btn("close", KeyCode::Enter, |btn| {
                            btn.throwing(()).right_aligned()
                        });
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

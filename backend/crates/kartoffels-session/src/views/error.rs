use crate::Background;
use anyhow::{Error, Result};
use kartoffels_ui::{Button, KeyCode, Term, UiWidget};
use ratatui::layout::{Constraint, Layout};
use ratatui::widgets::Paragraph;
use tracing::debug;

pub async fn run(term: &mut Term, bg: &Background, err: Error) -> Result<()> {
    debug!(?err, "run()");

    let err = Paragraph::new(err.to_string()).wrap(Default::default());

    loop {
        let go_back = term
            .frame(|ui| {
                let width = 50;
                let height = err.line_count(width) as u16 + 2;

                bg.render(ui);

                ui.error_window(width, height, Some(" ouch "), |ui| {
                    let [text_area, _, footer_area] = Layout::vertical([
                        Constraint::Fill(1),
                        Constraint::Length(1),
                        Constraint::Length(1),
                    ])
                    .areas(ui.area);

                    ui.clamp(text_area, |ui| {
                        ui.render(&err);
                    });

                    ui.clamp(footer_area, |ui| {
                        Button::new(KeyCode::Enter, "close")
                            .throwing(())
                            .right_aligned()
                            .render(ui);
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

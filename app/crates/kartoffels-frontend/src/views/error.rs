use crate::Background;
use anyhow::{Error, Result};
use kartoffels_ui::{Button, KeyCode, Term};
use ratatui::layout::{Constraint, Layout};
use ratatui::widgets::Paragraph;
use tracing::debug;

pub async fn run(term: &mut Term, bg: &Background, err: Error) -> Result<()> {
    debug!(?err, "run()");

    let err =
        Paragraph::new(err.to_string().to_lowercase()).wrap(Default::default());

    loop {
        let go_back = term
            .frame(|ui| {
                let width = 60;
                let height = err.line_count(width) as u16 + 2;

                ui.widget(bg);

                ui.error_window(width, height, Some(" ouch "), |ui| {
                    let [text_area, _, footer_area] = Layout::vertical([
                        Constraint::Fill(1),
                        Constraint::Length(1),
                        Constraint::Length(1),
                    ])
                    .areas(ui.area);

                    ui.widget_at(text_area, &err);

                    ui.widget_at(footer_area, {
                        Button::new(KeyCode::Enter, "close")
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

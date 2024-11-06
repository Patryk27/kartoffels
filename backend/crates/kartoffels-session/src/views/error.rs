use crate::Background;
use anyhow::{Error, Result};
use kartoffels_ui::{Button, RectExt, Render, Term};
use ratatui::widgets::{Paragraph, WidgetRef};
use termwiz::input::KeyCode;
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
                    err.render_ref(ui.area, ui.buf);

                    ui.clamp(ui.area.footer(1), |ui| {
                        Button::new(KeyCode::Enter, "got it")
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

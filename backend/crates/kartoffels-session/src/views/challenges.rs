use crate::Background;
use anyhow::Result;
use kartoffels_ui::{theme, Button, Term};
use termwiz::input::KeyCode;

pub async fn run(term: &mut Term, bg: &mut Background) -> Result<()> {
    loop {
        let go_back = term
            .draw(|ui| {
                bg.render(ui);

                ui.window(48, 3, Some(" challenges "), theme::YELLOW, |ui| {
                    ui.line("challenges not yet implemented, come back later!");
                    ui.space(1);

                    Button::new(KeyCode::Enter, "got it")
                        .throwing(())
                        .right_aligned()
                        .render(ui);
                });

                ui.catch::<()>()
            })
            .await?
            .flatten()
            .is_some();

        term.poll().await?;

        if go_back {
            return Ok(());
        }
    }
}

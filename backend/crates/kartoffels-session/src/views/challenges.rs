use crate::drivers::challenges::{Challenge, CHALLENGES};
use crate::Background;
use anyhow::Result;
use kartoffels_ui::{theme, Button, Render, Term};
use termwiz::input::KeyCode;
use tracing::debug;

pub async fn run(term: &mut Term, bg: &Background) -> Result<Response> {
    debug!("run()");

    loop {
        let resp = term
            .draw(|ui| {
                bg.render(ui);

                ui.window(48, 3, Some(" challenges "), theme::YELLOW, |ui| {
                    for (idx, challenge) in CHALLENGES.iter().enumerate() {
                        let key = KeyCode::Char((b'1' + (idx as u8)) as char);

                        Button::new(key, challenge.name)
                            .throwing(Response::Play(challenge))
                            .render(ui);

                        ui.line(challenge.desc);
                    }

                    ui.space(1);

                    Button::new(KeyCode::Escape, "go back")
                        .throwing(Response::GoBack)
                        .centered()
                        .render(ui);
                });
            })
            .await?;

        term.poll().await?;

        if let Some(resp) = resp {
            return Ok(resp);
        }
    }
}

#[derive(Debug)]
pub enum Response {
    Play(&'static Challenge),
    GoBack,
}

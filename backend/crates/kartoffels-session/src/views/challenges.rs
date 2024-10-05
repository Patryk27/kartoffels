use crate::drivers::challenges::{Challenge, CHALLENGES};
use crate::Background;
use anyhow::Result;
use kartoffels_ui::{Button, Render, Term};
use ratatui::layout::Rect;
use ratatui::widgets::{Paragraph, Wrap};
use termwiz::input::KeyCode;
use tracing::debug;

pub async fn run(term: &mut Term, bg: &Background) -> Result<Response> {
    debug!("run()");

    loop {
        let resp = term
            .draw(|ui| {
                bg.render(ui);

                let width = 48;

                let height = {
                    let mut height = 0;

                    for challenge in CHALLENGES {
                        height += 1;

                        height += Paragraph::new(challenge.desc)
                            .wrap(Wrap::default())
                            .line_count(width - 2)
                            as u16;

                        height += 1;
                    }

                    height + 1
                };

                ui.info_window(width, height, Some(" challenges "), |ui| {
                    for (idx, challenge) in CHALLENGES.iter().enumerate() {
                        let key = KeyCode::Char((b'1' + (idx as u8)) as char);

                        Button::new(key, challenge.name)
                            .throwing(Response::Play(challenge))
                            .render(ui);

                        let desc_area = Rect {
                            x: ui.area().x + 2,
                            ..ui.area()
                        };

                        let desc_height =
                            ui.clamp(desc_area, |ui| ui.line(challenge.desc));

                        ui.space(desc_height);
                        ui.space(1);
                    }

                    Button::new(KeyCode::Escape, "go back")
                        .throwing(Response::GoBack)
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

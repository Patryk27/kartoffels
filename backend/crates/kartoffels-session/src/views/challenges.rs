use crate::drivers::challenges::{Challenge, CHALLENGES};
use crate::Background;
use anyhow::Result;
use kartoffels_ui::{Button, Render, Term};
use ratatui::style::Stylize;
use ratatui::widgets::{Paragraph, Wrap};
use termwiz::input::KeyCode;
use tracing::debug;

pub async fn run(term: &mut Term, bg: &Background) -> Result<Response> {
    debug!("run()");

    let mut selected: Option<&Challenge> = None;

    loop {
        let event = term
            .draw(|ui| {
                bg.render(ui);

                let width = 48;

                let height = {
                    let mut h = CHALLENGES.len() as u16 + 2;

                    if let Some(selected) = selected {
                        let desc_h = Paragraph::new(selected.desc)
                            .wrap(Wrap::default())
                            .line_count(width);

                        h += 2 + (desc_h as u16);
                    }

                    h
                };

                ui.info_window(width, height, Some(" challenges "), |ui| {
                    for (idx, challenge) in CHALLENGES.iter().enumerate() {
                        let key = KeyCode::Char((b'1' + (idx as u8)) as char);

                        Button::new(key, challenge.name)
                            .throwing(Event::Select(challenge))
                            .render(ui);
                    }

                    ui.space(1);

                    if let Some(selected) = selected {
                        ui.line(format!("{}:", selected.name).bold());
                        ui.line(selected.desc);
                        ui.space(1);
                    }

                    ui.row(|ui| {
                        Button::new(KeyCode::Escape, "go back")
                            .throwing(Event::GoBack)
                            .render(ui);

                        if let Some(selected) = selected {
                            Button::new(KeyCode::Enter, "play")
                                .throwing(Event::Play(selected))
                                .right_aligned()
                                .render(ui);
                        }
                    });
                });
            })
            .await?;

        term.poll().await?;

        if let Some(event) = event {
            match event {
                Event::Select(challenge) => {
                    selected = Some(challenge);
                }
                Event::Play(challenge) => {
                    return Ok(Response::Play(challenge));
                }
                Event::GoBack => {
                    return Ok(Response::GoBack);
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum Response {
    Play(&'static Challenge),
    GoBack,
}

#[derive(Debug)]
enum Event {
    Select(&'static Challenge),
    Play(&'static Challenge),
    GoBack,
}

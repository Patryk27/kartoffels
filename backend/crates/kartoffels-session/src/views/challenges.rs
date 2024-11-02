use crate::drivers::challenges::{Challenge, CHALLENGES};
use crate::Background;
use anyhow::Result;
use kartoffels_store::Store;
use kartoffels_ui::{theme, Button, Fade, FadeDir, Render, Term};
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::Text;
use ratatui::widgets::{Paragraph, Wrap};
use termwiz::input::KeyCode;
use tracing::debug;

pub async fn run(
    store: &Store,
    term: &mut Term,
    bg: &Background,
    fade_in: bool,
) -> Result<Response> {
    debug!("run()");

    let mut fade_in = if fade_in && !store.testing() {
        Some(Fade::new(FadeDir::In))
    } else {
        None
    };

    let mut fade_out: Option<(Fade, Response)> = None;

    loop {
        let resp = term
            .draw(|ui| {
                bg.render(ui);

                let width = (ui.area().width - 2).min(60);

                // TODO ugh, doing manual layouting sometimes sucks
                let height = {
                    let mut height = 0;

                    for challenge in CHALLENGES {
                        height += 1;

                        height += Paragraph::new(challenge.desc)
                            .wrap(Wrap::default())
                            .line_count(width - 4);

                        height += 1;
                    }

                    (height + 1) as u16
                };

                ui.info_window(width, height, Some(" challenges "), |ui| {
                    for (idx, challenge) in CHALLENGES.iter().enumerate() {
                        let key = KeyCode::Char((b'1' + (idx as u8)) as char);

                        Button::new(key, challenge.name)
                            .throwing(Response::Play(challenge))
                            .render(ui);

                        let desc_area = Rect {
                            x: ui.area().x + 4,
                            ..ui.area()
                        };

                        let desc_height = ui.clamp(desc_area, |ui| {
                            ui.line(Text::raw(challenge.desc).fg(theme::GRAY))
                        });

                        ui.space(desc_height);
                        ui.space(1);
                    }

                    Button::new(KeyCode::Escape, "go back")
                        .throwing(Response::GoBack)
                        .render(ui);
                });

                if let Some(fade) = &fade_in {
                    if fade.render(ui).is_completed() {
                        fade_in = None;
                    }
                }

                if let Some((fade, _)) = &fade_out {
                    fade.render(ui);
                }
            })
            .await?;

        term.poll().await?;

        if let Some((fade, resp)) = &fade_out {
            if fade.is_completed() {
                return Ok(resp.clone());
            }

            continue;
        }

        if let Some(resp) = resp {
            if resp.fade_out() && !store.testing() {
                fade_out = Some((Fade::new(FadeDir::Out), resp));
            } else {
                return Ok(resp);
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum Response {
    Play(&'static Challenge),
    GoBack,
}

impl Response {
    fn fade_out(&self) -> bool {
        matches!(self, Response::Play(_))
    }
}

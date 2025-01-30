mod ctrls;

use self::ctrls::*;
use crate::views::game;
use crate::Background;
use anyhow::Result;
use kartoffels_store::{Session, Store};
use kartoffels_ui::{Button, Fade, FadeDir, Frame, KeyCode, UiWidget};
use ratatui::widgets::{Paragraph, Wrap};
use tracing::debug;

pub async fn run(
    store: &Store,
    sess: &Session,
    frame: &mut Frame,
    bg: &Background,
) -> Result<()> {
    let mut fade_in = false;

    loop {
        match run_once(store, frame, bg, fade_in).await? {
            Event::Play(challenge) => {
                game::run(store, sess, frame, |game| {
                    (challenge.run)(store, game)
                })
                .await?;

                fade_in = true;
            }

            Event::GoBack => {
                return Ok(());
            }
        }
    }
}

async fn run_once(
    store: &Store,
    frame: &mut Frame,
    bg: &Background,
    fade_in: bool,
) -> Result<Event> {
    debug!("run()");

    let mut fade_in = if fade_in && !store.testing() {
        Some(Fade::new(FadeDir::In))
    } else {
        None
    };

    let mut fade_out: Option<(Fade, Event)> = None;

    loop {
        let event = frame
            .update(|ui| {
                bg.render(ui);

                let width = (ui.area.width - 2).min(60);

                // TODO doing manual layouting sucks sometimes
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
                    for chl in CHALLENGES {
                        Button::new(chl.key, chl.name)
                            .help(chl.desc)
                            .throwing(Event::Play(chl))
                            .render(ui);

                        ui.space(1);
                    }

                    Button::new(KeyCode::Escape, "go-back")
                        .throwing(Event::GoBack)
                        .render(ui);
                });

                if let Some(fade) = &fade_in
                    && fade.render(ui).is_completed()
                {
                    fade_in = None;
                }

                if let Some((fade, _)) = &fade_out {
                    fade.render(ui);
                }
            })
            .await?;

        if let Some((fade, event)) = &fade_out {
            if fade.is_completed() {
                return Ok(*event);
            }

            continue;
        }

        if let Some(event) = event {
            if event.fade_out() && !store.testing() {
                fade_out = Some((Fade::new(FadeDir::Out), event));
            } else {
                return Ok(event);
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Event {
    Play(&'static Challenge),
    GoBack,
}

impl Event {
    fn fade_out(&self) -> bool {
        matches!(self, Event::Play(_))
    }
}

mod ctrls;

use self::ctrls::*;
use crate::views::game;
use crate::{BgMap, FadeCtrl, FadeCtrlEvent, Frame, UiWidget};
use anyhow::Result;
use kartoffels_store::{Session, Store};
use ratatui::widgets::{Paragraph, Wrap};
use termwiz::input::KeyCode;
use tracing::debug;

pub async fn run(
    store: &Store,
    sess: &Session,
    frame: &mut Frame,
    bg: &BgMap,
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
    bg: &BgMap,
    fade_in: bool,
) -> Result<Event> {
    debug!("run()");

    let mut fade = FadeCtrl::default()
        .animate(!store.testing())
        .fade_in(fade_in);

    loop {
        let event = frame
            .tick(|ui| {
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

                fade.render(ui, |ui| {
                    bg.render(ui);

                    ui.imodal(width, height, Some(" challenges "), |ui| {
                        for chl in CHALLENGES {
                            ui.btn(chl.name, chl.key, |btn| {
                                btn.help(chl.desc).throwing(Event::Play(chl))
                            });

                            ui.space(1);
                        }

                        ui.btn("exit", KeyCode::Escape, |btn| {
                            btn.throwing(Event::GoBack)
                        });
                    });
                });
            })
            .await?;

        if let Some(event) = event {
            return Ok(event);
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Event {
    Play(&'static Challenge),
    GoBack,
}

impl FadeCtrlEvent for Event {
    fn needs_fade_out(&self) -> bool {
        matches!(self, Event::Play(_))
    }
}

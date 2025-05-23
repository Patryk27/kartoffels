mod ctrls;

use self::ctrls::*;
use crate::views::game;
use crate::views::index::WINDOW_WIDTH;
use crate::{BgMap, Fade, Frame, Ui, UiWidget};
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
        debug!("run()");

        match main(store, frame, bg, fade_in).await? {
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

async fn main(
    store: &Store,
    frame: &mut Frame,
    bg: &BgMap,
    fade_in: bool,
) -> Result<Event> {
    let mut fade = Fade::new(store, fade_in);
    let mut view = View;

    loop {
        let event = frame
            .render(|ui| {
                bg.render(ui);
                view.render(ui);
                fade.render(ui);
            })
            .await?;

        if let Some(event @ Event::Play(_)) = event {
            fade.out(event);
            continue;
        }

        if let Some(event) = fade.poll().or(event) {
            return Ok(event);
        }
    }
}

#[derive(Debug)]
struct View;

impl View {
    fn render(&mut self, ui: &mut Ui<Event>) {
        let width = WINDOW_WIDTH;
        let height = self.height();

        ui.imodal(width, height, Some("challenges"), |ui| {
            for chl in CHALLENGES {
                ui.btn(chl.name, chl.key, |btn| {
                    btn.help(chl.desc).throwing(Event::Play(chl))
                });

                ui.space(1);
            }

            ui.btn("exit", KeyCode::Escape, |btn| btn.throwing(Event::GoBack));
        });
    }

    fn height(&self) -> u16 {
        let width = WINDOW_WIDTH;
        let mut height = 2;

        for (idx, chl) in CHALLENGES.iter().enumerate() {
            if idx > 0 {
                height += 1;
            }

            height += Paragraph::new(chl.desc)
                .wrap(Wrap::default())
                .line_count(width - 4);

            height += 1;
        }

        height as u16
    }
}

#[derive(Debug)]
enum Event {
    Play(&'static Challenge),
    GoBack,
}

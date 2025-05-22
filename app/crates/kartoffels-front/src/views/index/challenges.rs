mod ctrls;

use self::ctrls::*;
use crate::views::game;
use crate::views::index::WINDOW_WIDTH;
use crate::{theme, BgMap, FadeCtrl, FadeCtrlEvent, Frame, Ui, UiWidget};
use anyhow::Result;
use kartoffels_store::{Session, Store};
use ratatui::style::Stylize;
use ratatui::text::Text;
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

    let mut fade = FadeCtrl::new(store, fade_in);
    let mut view = View;

    loop {
        let event = frame
            .render(|ui| {
                fade.render(ui, |ui| {
                    bg.render(ui);
                    view.render(ui);
                });
            })
            .await?;

        if let Some(event) = event {
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

        ui.imodal(width, height, Some(" challenges "), |ui| {
            ui.line(
                Text::raw(
                    "challenges are single-player exercises where you have to \
                     implement a firmware that solves a specific problem",
                )
                .fg(theme::GRAY),
            );

            ui.space(1);

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
        let mut height = 5;

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

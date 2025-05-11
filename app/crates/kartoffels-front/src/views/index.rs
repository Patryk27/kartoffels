mod challenges;
mod play;
mod sandbox;
mod tutorial;
mod widgets;

use self::widgets::*;
use crate::{BgMap, FadeCtrl, FadeCtrlEvent, Frame};
use anyhow::Result;
use kartoffels_store::{Session, Store, WorldVis};
use ratatui::layout::{Constraint, Layout};
use tracing::debug;

pub async fn run(
    store: &Store,
    sess: &Session,
    frame: &mut Frame,
    bg: &BgMap,
) -> Result<()> {
    let mut fade_in = true;

    loop {
        match run_once(store, frame, bg, fade_in).await? {
            Event::Play => {
                play::run(store, sess, frame, bg).await?;
                fade_in = false;
            }

            Event::Sandbox => {
                sandbox::run(store, sess, frame, bg).await?;
                fade_in = false;
            }

            Event::Tutorial => {
                tutorial::run(store, sess, frame).await?;
                fade_in = true;
            }

            Event::Challenges => {
                challenges::run(store, sess, frame, bg).await?;
                fade_in = false;
            }

            Event::Quit => {
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

    let has_public_worlds =
        !store.find_worlds(WorldVis::Public).await?.is_empty();

    let mut fade = FadeCtrl::default()
        .animate(!store.testing())
        .fade_in(fade_in);

    loop {
        let event = frame
            .tick(|ui| {
                fade.render(ui, |ui| {
                    let [_, area, _] = Layout::horizontal([
                        Constraint::Fill(1),
                        Constraint::Length(Header::width()),
                        Constraint::Fill(1),
                    ])
                    .areas(ui.area);

                    let [_, header_area, _, menu_area, _] = Layout::vertical([
                        Constraint::Fill(1),
                        Constraint::Length(Header::height()),
                        Constraint::Fill(1),
                        Constraint::Length(Menu::height(ui, has_public_worlds)),
                        Constraint::Fill(1),
                    ])
                    .areas(area);

                    let [_, menu_area, _] = Layout::horizontal([
                        Constraint::Fill(1),
                        Constraint::Length(Menu::width()),
                        Constraint::Fill(1),
                    ])
                    .areas(menu_area);

                    ui.add(bg);

                    ui.clamp(header_area, |ui| {
                        Header::render(ui);
                    });

                    ui.clamp(menu_area, |ui| {
                        Menu::render(ui, has_public_worlds);
                    });

                    Footer::render(store, ui);
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
    Play,
    Sandbox,
    Tutorial,
    Challenges,
    Quit,
}

impl FadeCtrlEvent for Event {
    fn needs_fade_out(&self) -> bool {
        matches!(self, Event::Tutorial)
    }
}

mod challenges;
mod play;
mod sandbox;
mod tutorial;
mod widgets;

use self::widgets::*;
use crate::{BgMap, Fade, Frame};
use anyhow::Result;
use kartoffels_store::{Session, Store, WorldVis};
use ratatui::layout::{Constraint, Layout};
use tracing::info;

pub async fn run(
    store: &Store,
    sess: &Session,
    frame: &mut Frame,
    bg: &BgMap,
) -> Result<()> {
    let mut fade_in = true;

    loop {
        info!("run()");

        match main(store, frame, bg, fade_in).await? {
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

async fn main(
    store: &Store,
    frame: &mut Frame,
    bg: &BgMap,
    fade_in: bool,
) -> Result<Event> {
    let has_public_worlds =
        !store.find_worlds(WorldVis::Public).await?.is_empty();

    let mut fade = Fade::new(store, fade_in);

    loop {
        let event = frame
            .render(|ui| {
                let (menu, menu_size) = Menu::new(ui, has_public_worlds);

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
                    Constraint::Length(menu_size.height),
                    Constraint::Fill(1),
                ])
                .areas(area);

                let [_, menu_area, _] = Layout::horizontal([
                    Constraint::Fill(1),
                    Constraint::Length(menu_size.width),
                    Constraint::Fill(1),
                ])
                .areas(menu_area);

                ui.add(bg);

                ui.at(header_area, |ui| {
                    Header::render(ui);
                });

                ui.at(menu_area, |ui| {
                    menu.render(ui);
                });

                Footer::render(store, ui);

                fade.render(ui);
            })
            .await?;

        if let Some(event @ Event::Tutorial) = event {
            fade.out(event);
            continue;
        }

        if let Some(event) = fade.poll().or(event) {
            return Ok(event);
        }
    }
}

#[derive(Debug)]
enum Event {
    Play,
    Sandbox,
    Tutorial,
    Challenges,
    Quit,
}

mod challenges;
mod console;
mod play;
mod sandbox;
mod tutorial;
mod widgets;

use self::widgets::*;
use crate::Background;
use anyhow::Result;
use kartoffels_store::{Session, Store};
use kartoffels_ui::{FadeCtrl, FadeCtrlEvent, Frame, KeyCode, Modifiers};
use ratatui::layout::{Constraint, Layout};
use tracing::debug;

pub async fn run(
    store: &Store,
    sess: &Session,
    frame: &mut Frame,
    bg: &Background,
) -> Result<()> {
    let mut fade_in = true;

    loop {
        match run_once(store, frame, bg, fade_in).await? {
            Event::Admin => {
                console::run(store, sess, frame, bg).await?;
                fade_in = false;
            }

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
    bg: &Background,
    fade_in: bool,
) -> Result<Event> {
    debug!("run()");

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
                        Constraint::Length(Menu::height(store, ui)),
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
                        Menu::render(store, ui);
                    });

                    Footer::render(store, ui);

                    if ui.key(KeyCode::Char('x'), Modifiers::ALT) {
                        ui.throw(Event::Admin);
                    }
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
    Admin,
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

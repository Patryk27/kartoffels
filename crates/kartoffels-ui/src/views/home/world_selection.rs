use crate::{theme, Button, Clear, Term};
use anyhow::Result;
use kartoffels_store::Store;
use kartoffels_world::prelude::Handle as WorldHandle;
use ratatui::text::Line;
use ratatui::widgets::Widget;
use termwiz::input::KeyCode;
use tokio::time;

pub async fn run(term: &mut Term, store: &Store) -> Result<Response> {
    loop {
        let mut response = None;

        term.draw(|ui| {
            Clear::render(ui);

            let menu = build_menu(store);

            let width = menu
                .iter()
                .map(|item| item.width())
                .max()
                .unwrap_or_default();

            let height = menu.len() as u16;

            ui.info_dialog(width, height, Some(" play "), |ui| {
                for item in menu {
                    match item {
                        MenuItem::Line(item) => {
                            item.render(ui.area(), ui.buf());
                        }

                        MenuItem::Button(item, idx) => {
                            if item.render(ui).pressed {
                                if let Some(idx) = idx {
                                    response = Some(Response::Play(
                                        store.worlds[idx as usize].1.clone(),
                                    ));
                                } else {
                                    response = Some(Response::Quit);
                                }
                            }
                        }
                    }

                    ui.fill(1);
                }
            });
        })
        .await?;

        if let Some(response) = response {
            time::sleep(theme::INTERACTION_TIME).await;

            return Ok(response);
        }

        term.tick().await?;
    }
}

fn build_menu(store: &Store) -> Vec<MenuItem> {
    let mut items = vec![
        MenuItem::Line(Line::raw("choose world:").centered()),
        MenuItem::Line(Line::raw("")),
    ];

    for (idx, (_, world)) in store.worlds.iter().enumerate() {
        let idx = idx as u8;

        let btn =
            Button::new(KeyCode::Char((b'1' + idx) as char), world.name())
                .centered();

        items.push(MenuItem::Button(btn, Some(idx)));
    }

    items.push(MenuItem::Line(Line::raw("")));

    items.push(MenuItem::Button(
        Button::new(KeyCode::Escape, "go back").centered(),
        None,
    ));

    items
}

#[derive(Debug)]
enum MenuItem<'a> {
    Line(Line<'a>),
    Button(Button<'a>, Option<u8>),
}

impl MenuItem<'_> {
    fn width(&self) -> u16 {
        match self {
            MenuItem::Line(this) => this.width() as u16,
            MenuItem::Button(this, _) => this.width(),
        }
    }
}

#[derive(Debug)]
pub enum Response {
    Play(WorldHandle),
    Quit,
}

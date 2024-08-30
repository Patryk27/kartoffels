use crate::{Button, Clear, LayoutExt, Prompt, Term};
use anyhow::Result;
use kartoffels_store::Store;
use kartoffels_world::prelude::Handle as WorldHandle;
use ratatui::layout::Layout;
use ratatui::text::Line;
use ratatui::widgets::Widget;
use termwiz::input::KeyCode;

pub async fn run(term: &mut Term, store: &Store) -> Result<Outcome> {
    let mut prompt = Prompt::default();

    loop {
        let mut outcome = None;

        term.draw(|ui| {
            Clear::render(ui);

            let menu = build_menu(store);

            let area = {
                let width = menu
                    .iter()
                    .map(|item| item.width())
                    .max()
                    .unwrap_or_default();

                let height = menu.len() as u16;

                Layout::dialog(width, height, ui.area())
            };

            ui.clamp(area, |ui| {
                for item in menu {
                    match item {
                        MenuItem::Line(item) => {
                            item.render(ui.area(), ui.buf());
                        }

                        MenuItem::Button(item, idx) => {
                            if item.render(ui).activated {
                                if let Some(idx) = idx {
                                    outcome = Some(Outcome::Play(
                                        store.worlds[idx as usize].1.clone(),
                                    ));
                                } else {
                                    outcome = Some(Outcome::Quit);
                                }
                            }
                        }
                    }

                    ui.step(1);
                }

                ui.step(1);
                prompt.render(ui);
            });
        })
        .await?;

        if let Some(outcome) = outcome {
            return Ok(outcome);
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

        let btn = Button::new(
            KeyCode::Char((b'1' + idx) as char),
            world.name(),
            true,
        );

        items.push(MenuItem::Button(btn, Some(idx)));
    }

    items.push(MenuItem::Line(Line::raw("")));

    items.push(MenuItem::Button(
        Button::new(KeyCode::Escape, "go back", true),
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
pub enum Outcome {
    Play(WorldHandle),
    Quit,
}

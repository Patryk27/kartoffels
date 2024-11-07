mod bot_count;
mod bot_location;
mod bot_prefab;

pub use self::bot_count::*;
pub use self::bot_location::*;
pub use self::bot_prefab::*;
use super::Event as ParentEvent;
use kartoffels_ui::{Button, Render, Ui};
use termwiz::input::KeyCode;

#[derive(Debug, Default)]
pub struct SpawnPrefabBotDialog {
    focus: Option<Focus>,
    bot_count: BotCount,
    bot_prefab: BotPrefab,
    bot_location: BotLocation,
}

impl SpawnPrefabBotDialog {
    pub fn render(&mut self, ui: &mut Ui<ParentEvent>) {
        let event = ui.catch(|ui| {
            let width = 50;
            let height = self.height();
            let title = self.title();

            ui.info_window(width, height, Some(title), |ui| {
                self.render_body(ui);
                self.render_footer(ui);
            });
        });

        if let Some(event) = event {
            if let Some(event) = self.handle(event) {
                ui.throw(event);
            }
        }
    }

    fn title(&self) -> &'static str {
        match &self.focus {
            Some(Focus::BotCount) => " spawn-prefab › choose-count ",
            Some(Focus::BotPrefab) => " spawn-prefab › choose-prefab ",
            Some(Focus::BotLocation) => " spawn-prefab › choose-location ",
            None => " spawn-prefab ",
        }
    }

    fn height(&self) -> u16 {
        let body = match &self.focus {
            Some(Focus::BotCount) => BotCount::height(),
            Some(Focus::BotPrefab) => BotPrefab::height(),
            Some(Focus::BotLocation) => BotLocation::height(),

            None => {
                if let BotLocation::Random = &self.bot_location {
                    3
                } else {
                    2
                }
            }
        };

        body + 2
    }

    fn render_body(&self, ui: &mut Ui<Event>) {
        match &self.focus {
            Some(Focus::BotCount) => {
                BotCount::render_choice(ui);
            }
            Some(Focus::BotPrefab) => {
                BotPrefab::render_choice(ui);
            }
            Some(Focus::BotLocation) => {
                BotLocation::render_choice(ui);
            }

            None => {
                if let BotLocation::Random = &self.bot_location {
                    BotCount::render_focus(ui, &self.bot_count);
                }

                BotPrefab::render_focus(ui, &self.bot_prefab);
                BotLocation::render_focus(ui, &self.bot_location);
            }
        }
    }

    fn render_footer(&self, ui: &mut Ui<Event>) {
        ui.space(1);

        ui.row(|ui| {
            Button::new(KeyCode::Escape, "go-back")
                .throwing(Event::GoBack)
                .render(ui);

            if self.focus.is_none() {
                Button::new(KeyCode::Enter, "spawn")
                    .right_aligned()
                    .throwing(Event::Confirm)
                    .render(ui);
            }
        });
    }

    fn handle(&mut self, event: Event) -> Option<ParentEvent> {
        match event {
            Event::GoBack => {
                if self.focus.is_some() {
                    self.focus = None;
                } else {
                    return Some(ParentEvent::CloseDialog);
                }
            }

            Event::Confirm => {
                return Some(ParentEvent::SpawnPrefabBot {
                    count: self.bot_count,
                    prefab: self.bot_prefab,
                    location: self.bot_location,
                });
            }

            Event::FocusOn(val) => {
                self.focus = val;
            }

            Event::SetBotCount(val) => {
                self.bot_count = val;
                self.focus = None;
            }

            Event::SetBotPrefab(val) => {
                self.bot_prefab = val;
                self.focus = None;
            }

            Event::SetBotLocation(val) => {
                self.bot_location = val;
                self.focus = None;
            }
        }

        None
    }
}

#[derive(Debug)]
enum Event {
    GoBack,
    Confirm,
    FocusOn(Option<Focus>),
    SetBotCount(BotCount),
    SetBotPrefab(BotPrefab),
    SetBotLocation(BotLocation),
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
enum Focus {
    BotCount,
    BotPrefab,
    BotLocation,
}

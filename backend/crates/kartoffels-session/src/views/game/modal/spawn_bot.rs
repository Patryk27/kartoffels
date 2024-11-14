mod bot_count;
mod bot_position;
mod bot_source;

pub use self::bot_count::*;
pub use self::bot_position::*;
pub use self::bot_source::*;
use super::{Event as ParentEvent, UploadBotRequest};
use kartoffels_ui::{Button, Render, Ui};
use termwiz::input::KeyCode;

#[derive(Debug, Default)]
pub struct SpawnBotModal {
    focus: Option<Focus>,
    bot_source: BotSourceType,
    bot_position: BotPosition,
    bot_count: BotCount,
}

impl SpawnBotModal {
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
            Some(Focus::BotSource) => " spawn-bot › choose-source ",
            Some(Focus::BotPosition) => " spawn-bot › choose-position ",
            Some(Focus::BotCount) => " spawn-bot › choose-count ",
            None => " spawn-bot ",
        }
    }

    fn height(&self) -> u16 {
        let body = match &self.focus {
            Some(Focus::BotSource) => BotSourceType::height(),
            Some(Focus::BotPosition) => BotPosition::height(),
            Some(Focus::BotCount) => BotCount::height(),

            None => {
                if let BotPosition::Random = &self.bot_position {
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
            Some(Focus::BotSource) => {
                BotSourceType::render_choice(ui);
            }
            Some(Focus::BotPosition) => {
                BotPosition::render_choice(ui);
            }
            Some(Focus::BotCount) => {
                BotCount::render_choice(ui);
            }

            None => {
                BotSourceType::render_focus(ui, &self.bot_source);
                BotPosition::render_focus(ui, &self.bot_position);

                if let BotPosition::Random = &self.bot_position {
                    BotCount::render_focus(ui, &self.bot_count);
                }
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
                Button::new(KeyCode::Enter, "confirm")
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
                    return Some(ParentEvent::CloseModal);
                }
            }

            Event::Confirm => {
                return Some(ParentEvent::OpenUploadBotModal {
                    request: UploadBotRequest {
                        source: self.bot_source,
                        position: self.bot_position,
                        count: self.bot_count,
                    },
                });
            }

            Event::FocusOn(val) => {
                self.focus = val;
            }

            Event::SetBotSource(val) => {
                self.bot_source = val;
                self.focus = None;
            }

            Event::SetBotPosition(val) => {
                self.bot_position = val;
                self.focus = None;
            }

            Event::SetBotCount(val) => {
                self.bot_count = val;
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
    SetBotSource(BotSourceType),
    SetBotPosition(BotPosition),
    SetBotCount(BotCount),
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
enum Focus {
    BotSource,
    BotPosition,
    BotCount,
}

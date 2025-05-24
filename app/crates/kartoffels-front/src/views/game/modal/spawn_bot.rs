mod bot_count;
mod bot_position;
mod bot_source;

pub use self::bot_count::*;
pub use self::bot_position::*;
pub use self::bot_source::*;
use super::{Event as ParentEvent, UploadBotRequest};
use crate::Ui;
use termwiz::input::KeyCode;

#[derive(Debug)]
pub struct SpawnBotModal {
    focus: Option<Focus>,
    bot_source: BotSource,
    bot_position: BotPosition,
    bot_count: BotCount,
}

impl SpawnBotModal {
    pub fn new(bot_source: BotSource) -> Self {
        Self {
            focus: Default::default(),
            bot_source,
            bot_position: Default::default(),
            bot_count: Default::default(),
        }
    }

    pub fn render(&mut self, ui: &mut Ui<ParentEvent>) {
        let event = ui.catching(|ui| {
            let width = 50;
            let height = self.height();
            let title = self.title();

            ui.imodal(width, height, Some(title), |ui| {
                self.render_body(ui);
                self.render_footer(ui);
            });
        });

        if let Some(event) = event {
            self.handle(ui, event);
        }
    }

    fn title(&self) -> &'static str {
        match &self.focus {
            None => "spawn-bot",
            Some(Focus::BotSource) => "spawn-bot › choose-source",
            Some(Focus::BotPosition) => "spawn-bot › choose-position",
            Some(Focus::BotCount) => "spawn-bot › choose-count",
        }
    }

    fn height(&self) -> u16 {
        let body = match &self.focus {
            None => {
                if let BotPosition::Random = &self.bot_position {
                    3
                } else {
                    2
                }
            }

            Some(Focus::BotSource) => BotSource::height(),
            Some(Focus::BotPosition) => BotPosition::height(),
            Some(Focus::BotCount) => BotCount::height(),
        };

        body + 2
    }

    fn render_body(&self, ui: &mut Ui<Event>) {
        match &self.focus {
            None => {
                BotSource::render_btn(ui, &self.bot_source);
                BotPosition::render_btn(ui, &self.bot_position);

                if let BotPosition::Random = &self.bot_position {
                    BotCount::render_btn(ui, &self.bot_count);
                }
            }

            Some(Focus::BotSource) => {
                BotSource::render_form(ui);
            }
            Some(Focus::BotPosition) => {
                BotPosition::render_form(ui);
            }
            Some(Focus::BotCount) => {
                BotCount::render_form(ui);
            }
        }
    }

    fn render_footer(&self, ui: &mut Ui<Event>) {
        ui.space(1);

        ui.row(|ui| {
            ui.btn("exit", KeyCode::Escape, |btn| btn.throwing(Event::GoBack));

            if self.focus.is_none() {
                ui.btn("confirm", KeyCode::Enter, |btn| {
                    btn.right_aligned().throwing(Event::Confirm)
                });
            }
        });
    }

    fn handle(&mut self, ui: &mut Ui<ParentEvent>, event: Event) {
        match event {
            Event::GoBack => {
                if self.focus.is_some() {
                    self.focus = None;
                } else {
                    ui.throw(ParentEvent::CloseModal);
                }
            }

            Event::Confirm => {
                ui.throw(ParentEvent::OpenUploadBotModal {
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
    }
}

#[derive(Debug)]
enum Event {
    GoBack,
    Confirm,
    FocusOn(Option<Focus>),
    SetBotSource(BotSource),
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

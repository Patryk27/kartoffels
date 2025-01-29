mod bots;
mod error;
mod go_back;
mod help;
mod inspect_bot;
mod join_bot;
mod spawn_bot;
mod upload_bot;

pub use self::bots::*;
pub use self::error::*;
pub use self::go_back::*;
pub use self::help::*;
pub use self::inspect_bot::*;
pub use self::join_bot::*;
pub use self::spawn_bot::*;
pub use self::upload_bot::*;
use super::Event;
use kartoffels_store::Session;
use kartoffels_ui::{Backdrop, KeyCode, Modifiers, Ui};
use kartoffels_world::prelude::Snapshot;

#[allow(clippy::type_complexity)]
pub enum Modal {
    Bots(BotsModal),
    Error(ErrorModal),
    GoBack(GoBackModal),
    InspectBot(InspectBotModal),
    JoinBot(JoinBotModal),
    SpawnBot(SpawnBotModal),
    UploadBot(UploadBotModal),

    Help(HelpMsgRef),
    Custom(Box<dyn FnMut(&mut Ui<()>) + Send>),
}

impl Modal {
    pub fn render(
        &mut self,
        ui: &mut Ui<Event>,
        sess: &Session,
        world: &Snapshot,
    ) {
        Backdrop::render(ui);

        match self {
            Modal::Bots(this) => {
                this.render(ui, world);
            }
            Modal::Error(this) => {
                this.render(ui);
            }
            Modal::GoBack(this) => {
                this.render(ui);
            }
            Modal::InspectBot(this) => {
                this.render(ui, world);
            }
            Modal::JoinBot(this) => {
                this.render(ui, world);
            }
            Modal::SpawnBot(this) => {
                this.render(ui);
            }
            Modal::UploadBot(this) => {
                this.render(ui, sess);
            }

            Modal::Help(this) => {
                let event = ui.catch(|ui| {
                    this.render(ui);
                });

                if let Some(event) = event {
                    match event {
                        HelpMsgEvent::Copy(payload) => {
                            ui.copy(payload);
                        }
                        HelpMsgEvent::Close => {
                            ui.throw(Event::CloseModal);
                        }
                    }
                }

                if ui.key(KeyCode::Escape, Modifiers::NONE) {
                    ui.throw(Event::CloseModal);
                }
            }

            Modal::Custom(this) => {
                ui.catch(this);
            }
        }
    }
}

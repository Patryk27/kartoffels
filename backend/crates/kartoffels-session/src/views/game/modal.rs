mod bots;
mod error;
mod help;
mod inspect_bot;
mod join_bot;
mod menu;
mod spawn_bot;
mod upload_bot;

pub use self::bots::*;
pub use self::error::*;
pub use self::help::*;
pub use self::inspect_bot::*;
pub use self::join_bot::*;
pub use self::menu::*;
pub use self::spawn_bot::*;
pub use self::upload_bot::*;
use super::Event;
use kartoffels_store::SessionId;
use kartoffels_ui::{Backdrop, Ui};
use kartoffels_world::prelude::Snapshot;
use termwiz::input::{KeyCode, Modifiers};

#[allow(clippy::type_complexity)]
pub enum Modal {
    Bots(BotsModal),
    Error(ErrorModal),
    InspectBot(InspectBotModal),
    JoinBot(JoinBotModal),
    Menu(MenuModal),
    SpawnBot(SpawnBotModal),
    UploadBot(UploadBotModal),

    Help(HelpMsgRef),
    Custom(Box<dyn FnMut(&mut Ui<()>) + Send>),
}

impl Modal {
    pub fn render(
        &mut self,
        ui: &mut Ui<Event>,
        sess: SessionId,
        world: &Snapshot,
        can_restart: bool,
    ) {
        Backdrop::render(ui);

        match self {
            Modal::Bots(this) => {
                this.render(ui, world);
            }
            Modal::Error(this) => {
                this.render(ui);
            }
            Modal::InspectBot(this) => {
                this.render(ui, world);
            }
            Modal::JoinBot(this) => {
                this.render(ui, world);
            }
            Modal::Menu(this) => {
                this.render(ui, can_restart);
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
                        HelpMsgResponse::Copy(payload) => {
                            ui.copy(payload);
                        }
                        HelpMsgResponse::Close => {
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

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
use crate::{Backdrop, Ui};
use kartoffels_store::Session;
use kartoffels_world::prelude as w;
use termwiz::input::{KeyCode, Modifiers};

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
        world: &w::Snapshot,
    ) {
        Backdrop::render(ui);

        match self {
            Self::Bots(this) => {
                this.render(ui, world);
            }
            Self::Error(this) => {
                this.render(ui);
            }
            Self::GoBack(this) => {
                this.render(ui);
            }
            Self::InspectBot(this) => {
                this.render(ui, world);
            }
            Self::JoinBot(this) => {
                this.render(ui, world);
            }
            Self::SpawnBot(this) => {
                this.render(ui);
            }
            Self::UploadBot(this) => {
                this.render(ui, sess);
            }

            Self::Help(this) => {
                if let Some(event) = ui.catching(|ui| ui.add(&**this)) {
                    ui.throw(match event {
                        HelpMsgEvent::Copy { payload } => {
                            Event::Copy { payload }
                        }
                        HelpMsgEvent::Close => Event::CloseModal,
                    });
                }

                if ui.key(KeyCode::Escape, Modifiers::NONE) {
                    ui.throw(Event::CloseModal);
                }
            }

            Self::Custom(this) => {
                ui.catching(this);
            }
        }
    }
}

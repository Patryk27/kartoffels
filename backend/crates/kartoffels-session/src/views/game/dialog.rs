mod bots;
mod error;
mod help;
mod inspect_bot;
mod join_bot;
mod leaving;
mod spawn_bot;
mod upload_bot;

pub use self::bots::*;
pub use self::error::*;
pub use self::help::*;
pub use self::inspect_bot::*;
pub use self::join_bot::*;
pub use self::leaving::*;
pub use self::spawn_bot::*;
pub use self::upload_bot::*;
use super::Event;
use kartoffels_store::SessionId;
use kartoffels_ui::{Backdrop, Ui};
use kartoffels_world::prelude::Snapshot;
use termwiz::input::{KeyCode, Modifiers};

#[allow(clippy::type_complexity)]
pub enum Dialog {
    Bots(BotsDialog),
    Error(ErrorDialog),
    GoBack(GoBackDialog),
    InspectBot(InspectBotDialog),
    JoinBot(JoinBotDialog),
    SpawnBot(SpawnBotDialog),
    UploadBot(UploadBotDialog),

    Help(HelpMsgRef),
    Custom(Box<dyn FnMut(&mut Ui<()>) + Send>),
}

impl Dialog {
    pub fn render(
        &mut self,
        ui: &mut Ui<Event>,
        sess: SessionId,
        world: &Snapshot,
    ) {
        Backdrop::render(ui);

        match self {
            Dialog::Bots(this) => {
                this.render(ui, world);
            }
            Dialog::Error(this) => {
                this.render(ui);
            }
            Dialog::GoBack(this) => {
                this.render(ui);
            }
            Dialog::InspectBot(this) => {
                this.render(ui, world);
            }
            Dialog::JoinBot(this) => {
                this.render(ui, world);
            }
            Dialog::SpawnBot(this) => {
                this.render(ui);
            }
            Dialog::UploadBot(this) => {
                this.render(ui, sess);
            }

            Dialog::Help(this) => {
                let event = ui.catch(|ui| {
                    this.render(ui);
                });

                if let Some(event) = event {
                    match event {
                        HelpMsgResponse::Copy(payload) => {
                            ui.copy(payload);
                        }
                        HelpMsgResponse::Close => {
                            ui.throw(Event::CloseDialog);
                        }
                    }
                }

                if ui.key(KeyCode::Escape, Modifiers::NONE) {
                    ui.throw(Event::CloseDialog);
                }
            }

            Dialog::Custom(this) => {
                ui.catch(this);
            }
        }
    }
}

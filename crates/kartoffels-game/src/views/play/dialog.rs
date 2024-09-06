mod bots;
mod configure_world;
mod error;
mod help;
mod join_bot;
mod upload_bot;

pub use self::bots::*;
pub use self::configure_world::*;
pub use self::error::*;
pub use self::help::*;
pub use self::join_bot::*;
pub use self::upload_bot::*;
use super::State;
use anyhow::Result;
use kartoffels_ui::{Backdrop, Ui};
use kartoffels_world::prelude::{BotId, Snapshot};
use std::ops::ControlFlow;

pub enum Dialog {
    Bots(BotsDialog),
    ConfigureWorld(ConfigureWorldDialog),
    Custom(Box<dyn FnMut(&mut Ui) + Send + Sync>),
    Error(ErrorDialog),
    Help(HelpDialogRef),
    JoinBot(JoinBotDialog),
    UploadBot(UploadBotDialog),
}

impl Dialog {
    pub fn render(
        &mut self,
        ui: &mut Ui,
        world: &Snapshot,
    ) -> Option<DialogResponse> {
        Backdrop::render(ui);

        match self {
            Dialog::Bots(this) => this.render(ui, world),
            Dialog::ConfigureWorld(this) => this.render(ui),
            Dialog::Error(this) => this.render(ui),
            Dialog::JoinBot(this) => this.render(ui, world),
            Dialog::UploadBot(this) => this.render(ui),

            Dialog::Help(this) => match this.render(ui) {
                Some(HelpDialogResponse::Copy(payload)) => {
                    ui.copy(payload);
                    None
                }
                Some(HelpDialogResponse::Close) => Some(DialogResponse::Close),
                None => None,
            },

            Dialog::Custom(this) => {
                (this)(ui);
                None
            }
        }
    }
}

#[derive(Debug)]
pub enum DialogResponse {
    Close,
    JoinBot(BotId),
    UploadBot(String),
    Throw(String),
}

impl DialogResponse {
    pub async fn handle(
        self,
        state: &mut State,
    ) -> Result<ControlFlow<(), ()>> {
        match self {
            DialogResponse::Close => {
                state.dialog = None;
            }

            DialogResponse::JoinBot(id) => {
                state.dialog = None;
                state.join_bot(id);
            }

            DialogResponse::UploadBot(src) => {
                state.dialog = None;
                state.upload_bot(src).await?;
            }

            DialogResponse::Throw(err) => {
                state.dialog = Some(Dialog::Error(ErrorDialog::new(err)));
            }
        }

        Ok(ControlFlow::Continue(()))
    }
}

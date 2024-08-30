mod bots;
mod error;
mod help;
mod join_bot;
mod upload_bot;

pub use self::bots::*;
pub use self::error::*;
pub use self::help::*;
pub use self::join_bot::*;
pub use self::upload_bot::*;
use crate::Ui;
use kartoffels_world::prelude::{BotId, Snapshot};
use termwiz::input::{KeyCode, Modifiers};

#[derive(Debug)]
pub enum Dialog {
    Bots(BotsDialog),
    Error(ErrorDialog),
    Help(HelpDialog),
    JoinBot(JoinBotDialog),
    UploadBot(UploadBotDialog),
}

impl Dialog {
    pub fn render(
        &mut self,
        ui: &mut Ui,
        snapshot: &Snapshot,
    ) -> Option<DialogEvent> {
        if ui.key(KeyCode::Escape, Modifiers::NONE) {
            return Some(DialogEvent::Close);
        }

        match self {
            Dialog::Bots(this) => this.render(ui, snapshot),
            Dialog::Error(this) => this.render(ui),
            Dialog::Help(this) => this.render(ui),
            Dialog::JoinBot(this) => this.render(ui, snapshot),
            Dialog::UploadBot(this) => this.render(ui),
        }
    }
}

#[derive(Debug)]
pub enum DialogEvent {
    Close,
    JoinBot(BotId),
    UploadBot(String),
    OpenTutorial,
    Throw(String),
}

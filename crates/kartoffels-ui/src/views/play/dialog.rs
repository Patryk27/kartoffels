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
use kartoffels_world::prelude::{BotId, Update};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use termwiz::input::{InputEvent, KeyCode};

#[derive(Debug)]
pub enum Dialog {
    Bots(BotsDialog),
    Error(ErrorDialog),
    Help(HelpDialog),
    JoinBot(JoinBotDialog),
    UploadBot(UploadBotDialog),
}

impl Dialog {
    pub fn render(&mut self, area: Rect, buf: &mut Buffer, update: &Update) {
        match self {
            Dialog::Bots(this) => this.render(area, buf, update),
            Dialog::Error(this) => this.render(area, buf),
            Dialog::Help(this) => this.render(area, buf),
            Dialog::JoinBot(this) => this.render(area, buf),
            Dialog::UploadBot(this) => this.render(area, buf),
        }
    }

    pub fn handle(&mut self, event: InputEvent) -> Option<DialogEvent> {
        if let InputEvent::Key(event) = &event {
            if event.key == KeyCode::Escape {
                return Some(DialogEvent::Close);
            }
        }

        match self {
            Dialog::Bots(this) => this.handle(event),
            Dialog::Error(this) => this.handle(event),
            Dialog::Help(this) => this.handle(event),
            Dialog::JoinBot(this) => this.handle(event),
            Dialog::UploadBot(this) => this.handle(event),
        }
    }

    pub async fn tick(&mut self) {
        match self {
            Dialog::JoinBot(this) => this.tick().await,
            Dialog::UploadBot(this) => this.tick().await,
            _ => (),
        }
    }
}

#[derive(Debug)]
pub enum DialogEvent {
    Close,
    JoinBot(BotId),
    UploadBot(String),
    Throw(String),
}

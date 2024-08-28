mod bots;
mod error;
mod help;
mod upload;

pub use self::bots::*;
pub use self::error::*;
pub use self::help::*;
pub use self::upload::*;
use kartoffels_world::prelude::BotUpdate;
use ratatui::prelude::{Buffer, Rect};
use termwiz::input::{InputEvent, KeyCode};

#[derive(Debug)]
pub enum Dialog {
    Bots(BotsDialog),
    Error(ErrorDialog),
    Help,
    Upload(UploadDialog),
}

impl Dialog {
    pub fn handle(&mut self, event: InputEvent) -> DialogOutcome {
        if let InputEvent::Key(event) = &event {
            if event.key == KeyCode::Escape {
                return DialogOutcome::Close;
            }
        }

        match self {
            Dialog::Bots(this) => {
                this.handle(event);

                DialogOutcome::None
            }

            Dialog::Upload(this) => match this.handle(event) {
                UploadDialogOutcome::Ready(src) => DialogOutcome::Upload(src),
                UploadDialogOutcome::None => DialogOutcome::None,
            },

            _ => DialogOutcome::None,
        }
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, bots: &[BotUpdate]) {
        match self {
            Dialog::Bots(this) => {
                this.render(area, buf, bots);
            }
            Dialog::Error(this) => {
                this.render(area, buf);
            }
            Dialog::Help => {
                HelpDialog.render(area, buf);
            }
            Dialog::Upload(this) => {
                this.render(area, buf);
            }
        }
    }
}

#[derive(Debug)]
pub enum DialogOutcome {
    Close,
    Upload(String),
    None,
}

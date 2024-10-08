use kartoffels_ui::{Dialog, DialogButton};
use termwiz::input::KeyCode;

pub type HelpDialog = Dialog<HelpDialogResponse>;
pub type HelpDialogRef = &'static HelpDialog;

#[derive(Clone, Copy, Debug)]
pub enum HelpDialogResponse {
    Copy(&'static str),
    Close,
}

impl HelpDialogResponse {
    pub fn close() -> DialogButton<Self> {
        DialogButton::new(KeyCode::Escape, "close", Self::Close).right_aligned()
    }
}

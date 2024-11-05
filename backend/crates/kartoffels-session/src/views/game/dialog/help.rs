use kartoffels_ui::{Msg, MsgButton};
use termwiz::input::KeyCode;

pub type HelpMsg = Msg<HelpMsgResponse>;
pub type HelpMsgRef = &'static HelpMsg;

#[derive(Clone, Copy, Debug)]
pub enum HelpMsgResponse {
    Copy(&'static str),
    Close,
}

impl HelpMsgResponse {
    pub fn close() -> MsgButton<Self> {
        MsgButton::new(KeyCode::Escape, "close", Self::Close).right_aligned()
    }
}

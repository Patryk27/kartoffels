use kartoffels_ui::{KeyCode, Msg, MsgButton};

pub type HelpMsg = Msg<HelpMsgEvent>;
pub type HelpMsgRef = &'static HelpMsg;

#[derive(Clone, Copy, Debug)]
pub enum HelpMsgEvent {
    Copy(&'static str),
    Close,
}

impl HelpMsgEvent {
    pub fn close() -> MsgButton<Self> {
        MsgButton::new(KeyCode::Escape, "close", Self::Close).right_aligned()
    }
}

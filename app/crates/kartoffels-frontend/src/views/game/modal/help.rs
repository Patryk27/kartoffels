use kartoffels_ui::{KeyCode, Msg, MsgButton};

pub type HelpMsg = Msg<HelpMsgEvent>;
pub type HelpMsgRef = &'static HelpMsg;

#[derive(Clone, Debug)]
pub enum HelpMsgEvent {
    Copy { payload: String },
    Close,
}

impl HelpMsgEvent {
    pub fn close() -> MsgButton<Self> {
        MsgButton::new("close", KeyCode::Escape, Self::Close).right_aligned()
    }
}

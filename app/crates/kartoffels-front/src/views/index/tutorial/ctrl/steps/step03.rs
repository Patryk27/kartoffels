use super::prelude::*;
use termwiz::input::KeyCode;

const CMD: &str = "git clone https://github.com/patryk27/kartoffel";

static MSG: LazyLock<Msg<Action>> = LazyLock::new(|| Msg {
    title: Some(" tutorial (3/16) "),

    body: vec![
        MsgLine::new("look at you, learning so fast - *next lesson!*"),
        MsgLine::new(""),
        MsgLine::new("run this:"),
        MsgLine::new(format!("    {CMD}")),
        MsgLine::new(""),
        MsgLine::new("... and press [`enter`] once you're ready"),
    ],

    buttons: vec![
        MsgButton::escape("prev", Action::Prev),
        MsgButton::new("copy-command", KeyCode::Char('c'), Action::Copy)
            .right_aligned(),
        MsgButton::enter("next", Action::Next),
    ],
});

#[derive(Clone, Copy, Debug)]
enum Action {
    Prev,
    Copy,
    Next,
}

pub async fn run(ctxt: &mut TutorialCtxt) -> Result<bool> {
    loop {
        match ctxt.game.msg(&MSG).await? {
            Action::Prev => {
                return Ok(false);
            }
            Action::Copy => {
                ctxt.game.copy(CMD).await?;
            }
            Action::Next => {
                return Ok(true);
            }
        }
    }
}

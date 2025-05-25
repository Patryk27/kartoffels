use super::prelude::*;
use termwiz::input::KeyCode;

const CMD: &str = "git clone https://github.com/patryk27/kartoffel";

static MSG: LazyLock<Msg<Action>> = LazyLock::new(|| {
    Msg::new("tutorial (3/16)")
        .line("look at you, learning so fast - *next lesson!*")
        .line("")
        .line("run this:")
        .line(format!("    {CMD}"))
        .line("")
        .line("... and press [`enter`] once you're ready")
        .btn(MsgBtn::escape("back", Action::Prev))
        .btn(
            MsgBtn::new("copy-command", KeyCode::Char('c'), Action::Copy)
                .right_aligned(),
        )
        .btn(MsgBtn::enter("next", Action::Next))
        .build()
});

#[derive(Clone, Copy, Debug)]
enum Action {
    Prev,
    Copy,
    Next,
}

pub async fn run(ctxt: &mut TutorialCtxt) -> Result<bool> {
    info!("run()");

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

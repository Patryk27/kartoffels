use super::prelude::*;

static MSG: LazyLock<Msg<bool>> = LazyLock::new(|| Msg {
    title: Some("tutorial (4/16)"),

    body: vec![
        MsgLine::new("fabulous!").fg(theme::PINK).bold(),
        MsgLine::new(""),
        MsgLine::new(
            "now launch vscode, vim, emacs or whatever gives your life colors \
             and open `main.rs` from the cloned repository",
        ),
        MsgLine::new(""),
        MsgLine::new(
            "for, you see, writing a bot is similar to writing a normal \
             program - but it's also different, _mucho_ different",
        ),
        MsgLine::new(""),
        MsgLine::new(
            "there's no `std::fs`, no `std::net` - everything your bot has \
             is itself, a bit of memory, serial port, and a couple of other \
             peripherals",
        ),
        MsgLine::new(""),
        MsgLine::new("you know, like people in ancient rome did"),
    ],

    buttons: vec![
        MsgButton::escape("prev", false),
        MsgButton::enter("next", true),
    ],
});

pub async fn run(ctxt: &mut TutorialCtxt) -> Result<bool> {
    info!("run()");

    ctxt.game.msg(&MSG).await
}

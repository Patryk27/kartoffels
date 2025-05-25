use super::prelude::*;

static MSG: LazyLock<Msg<bool>> = LazyLock::new(|| {
    Msg::new("tutorial (4/16)")
        .line(MsgLine::new("*fabulous!*").fg(theme::PINK))
        .line("")
        .line(
            "now launch vscode, vim, emacs or whatever gives your life colors \
             and open `main.rs` from the cloned repository",
        )
        .line("")
        .line(
            "for, you see, writing a bot is similar to writing a normal \
             program - but it's also different, _mucho_ different",
        )
        .line("")
        .line(
            "there's no `std::fs`, no `std::net` - everything your bot has \
             is itself, a bit of memory, serial port, and a couple of other \
             peripherals",
        )
        .line("")
        .line("you know, like people in ancient rome did")
        .btn(MsgBtn::escape("back", false))
        .btn(MsgBtn::enter("next", true))
        .build()
});

pub async fn run(ctxt: &mut TutorialCtxt) -> Result<bool> {
    info!("run()");

    ctxt.game.msg(&MSG).await
}

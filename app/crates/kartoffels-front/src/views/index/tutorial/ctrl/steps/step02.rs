use super::prelude::*;

static MSG: LazyLock<Msg<bool>> = LazyLock::new(|| {
    Msg::new("tutorial (2/16)")
        .line("*lesson #1:*")
        .line("- you can navigate the interface using keyboard and/or mouse")
        .line(MsgLine::ssh(
            "  (including when you're connected through the terminal)",
        ))
        .line("")
        .line("*lesson #2, at any time:*")
        .line(MsgLine::ssh(
            "- press [`Ctrl-c`] to quit the game (no questions asked)",
        ))
        .line("")
        .line("*lesson #3, during the game:*")
        .line("- press [`h`] to get help (ambulance paid separately)")
        .line("- press [`w`/`a`/`s`/`d`] or arrow keys to move the camera")
        .line("")
        .line("*lesson #4:*")
        .line("- don't lick yellow snow")
        .btn(MsgBtn::escape("back", false))
        .btn(MsgBtn::enter("next", true))
        .build()
});

pub async fn run(ctxt: &mut TutorialCtxt) -> Result<bool> {
    info!("run()");

    ctxt.game.msg(&MSG).await
}

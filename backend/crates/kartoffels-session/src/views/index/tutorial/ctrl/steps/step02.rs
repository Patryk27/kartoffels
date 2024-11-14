use super::prelude::*;

static MSG: LazyLock<Msg> = LazyLock::new(|| Msg {
    title: Some(" tutorial (2/16) "),

    body: vec![
        MsgLine::new("lesson #1:").bold(),
        MsgLine::new(
            "- you can navigate the interface using keyboard and/or mouse",
        ),
        MsgLine::ssh(
            "  (that includes when you're connected through the terminal)",
        ),
        MsgLine::new(""),
        MsgLine::new("lesson #2, at any time:").bold(),
        MsgLine::new("- press [`Ctrl-a`] to go back to the main menu"),
        MsgLine::ssh("- press [`Ctrl-c`] to disconnect and leave the game"),
        MsgLine::new(""),
        MsgLine::new("lesson #3, during the game, when no message is visible:")
            .bold(),
        MsgLine::new("- press [`h`] to get help (ambulance paid separately)"),
        MsgLine::new(
            "- press [`w`/`a`/`s`/`d`] or arrow keys to move the camera",
        ),
        MsgLine::new(""),
        MsgLine::new("lesson #4:").bold(),
        MsgLine::new("- don't lick yellow snow"),
    ],

    buttons: vec![MsgButton::confirm("next", ())],
});

pub async fn run(ctxt: &mut TutorialCtxt) -> Result<()> {
    ctxt.game.msg(&MSG).await?;

    Ok(())
}

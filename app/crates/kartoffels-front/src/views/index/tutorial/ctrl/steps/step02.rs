use super::prelude::*;

static MSG: LazyLock<Msg<bool>> = LazyLock::new(|| Msg {
    title: Some(" tutorial (2/16) "),

    body: vec![
        MsgLine::new("lesson #1:").bold(),
        MsgLine::new(
            "- you can navigate the interface using keyboard and/or mouse",
        ),
        MsgLine::ssh(
            "  (including when you're connected through the terminal)",
        ),
        MsgLine::new(""),
        MsgLine::new("lesson #2, at any time:").bold(),
        MsgLine::ssh(
            "- press [`Ctrl-c`] to quit the game (no questions asked)",
        ),
        MsgLine::new(""),
        MsgLine::new("lesson #3, during the game:").bold(),
        MsgLine::new("- press [`h`] to get help (ambulance paid separately)"),
        MsgLine::new(
            "- press [`w`/`a`/`s`/`d`] or arrow keys to move the camera",
        ),
        MsgLine::new(""),
        MsgLine::new("lesson #4:").bold(),
        MsgLine::new("- don't lick yellow snow"),
    ],

    buttons: vec![
        MsgButton::escape("prev", false),
        MsgButton::enter("next", true),
    ],
});

pub async fn run(ctxt: &mut TutorialCtxt) -> Result<bool> {
    ctxt.game.msg(&MSG).await
}

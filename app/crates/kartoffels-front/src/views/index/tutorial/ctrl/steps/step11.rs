use super::prelude::*;

static MSG: LazyLock<Msg> = LazyLock::new(|| Msg {
    title: Some(" tutorial (11/16) "),

    body: vec![
        MsgLine::new("nice!"),
        MsgLine::new(""),
        MsgLine::new("i mean, not nice, because we're dead, but baby steps"),
        MsgLine::new(""),
        MsgLine::new(
            "now it's time for you to learn about *the radar* using which the \
             bot can recognize its environment",
        ),
    ],

    buttons: vec![MsgButton::enter("next", ())],
});

pub async fn run(ctxt: &mut TutorialCtxt) -> Result<()> {
    ctxt.sync().await?;
    ctxt.game.msg(&MSG).await?;

    Ok(())
}

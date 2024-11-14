use super::prelude::*;

static MSG: LazyLock<Msg> = LazyLock::new(|| Msg {
    title: Some(" tutorial (16/16) "),

    body: vec![
        MsgLine::new("✨ *yay, you made it!* ✨")
            .fg(theme::GREEN)
            .bold()
            .centered(),
        MsgLine::new(""),
        MsgLine::new(
            "not sure if your mom and dad are proud of you, but _i_ am !!",
        )
        .centered(),
        MsgLine::new(""),
        MsgLine::new("now, waste no time:").centered(),
        MsgLine::new("🥔 have fun and remember the power of potato 🥔")
            .centered(),
    ],

    buttons: vec![MsgButton::confirm("complete", ())],
});

pub async fn run(ctxt: &mut TutorialCtxt) -> Result<()> {
    ctxt.world.set_map(Default::default()).await?;
    ctxt.sync().await?;
    ctxt.game.msg(&MSG).await?;

    Ok(())
}

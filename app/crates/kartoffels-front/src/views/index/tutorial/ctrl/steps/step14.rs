use super::prelude::*;

static MSG: LazyLock<Msg> = LazyLock::new(|| Msg {
    title: Some("tutorial (14/16)"),

    body: vec![
        MsgLine::new("congrats!"),
        MsgLine::new(""),
        MsgLine::new(
            "i don't want to keep you for much longer, so let's wrap things up \
             with a lesson on the last peripheral you need to know in order to \
             play:",
        ),
        MsgLine::new(""),
        MsgLine::new("🔪 the knife 🔪")
            .centered()
            .fg(theme::YELLOW)
            .bold(),
    ],

    buttons: vec![MsgButton::enter("lets-take-a-stab-at-it", ())],
});

pub async fn run(ctxt: &mut TutorialCtxt) -> Result<()> {
    info!("run()");

    ctxt.delete_bots().await?;
    ctxt.sync().await?;
    ctxt.game.msg(&MSG).await
}

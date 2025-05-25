use super::prelude::*;

static MSG: LazyLock<Msg> = LazyLock::new(|| {
    Msg::new("tutorial (14/16)")
        .line("congrats!")
        .line("")
        .line(
            "i don't want to keep you for much longer, so let's wrap things up \
             with a lesson on the last peripheral you need to know in order to \
             play:",
        )
        .line("")
        .line(
            MsgLine::new("*🔪 the knife 🔪*")
                .centered()
                .fg(theme::YELLOW),
        )
        .btn(MsgBtn::enter("lets-take-a-stab-at-it", ()))
        .build()
});

pub async fn run(ctxt: &mut TutorialCtxt) -> Result<()> {
    info!("run()");

    ctxt.delete_bots().await?;
    ctxt.sync().await?;
    ctxt.game.msg(&MSG).await
}

use super::prelude::*;

static MSG: LazyLock<Msg> = LazyLock::new(|| {
    Msg::new("tutorial (11/16)")
        .line("nice!")
        .line("")
        .line("i mean, not nice, because we're dead, but baby steps")
        .line("")
        .line(
            "now it's time for you to learn about *the radar* using which the \
             bot can recognize its environment",
        )
        .btn(MsgBtn::enter("next", ()))
        .build()
});

pub async fn run(ctxt: &mut TutorialCtxt) -> Result<()> {
    info!("run()");

    ctxt.sync().await?;
    ctxt.game.msg(&MSG).await
}

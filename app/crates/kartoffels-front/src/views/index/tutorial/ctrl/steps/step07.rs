use super::prelude::*;

static MSG: LazyLock<Msg> = LazyLock::new(|| {
    Msg::new("tutorial (7/16)")
        .line(
            "anyway, close this message to resume the game and let's see the \
             bot in action",
        )
        .line("")
        .line(
            "if everything goes correctly, we should see the machine driving \
             in squares, *how exquisite*!",
        )
        .btn(MsgBtn::enter("next", ()))
        .build()
});

pub async fn run(ctxt: &mut TutorialCtxt) -> Result<()> {
    info!("run()");

    ctxt.game.msg(&MSG).await?;
    ctxt.game.resume().await?;

    time::sleep(Duration::from_secs(6)).await;

    ctxt.delete_bots().await?;
    ctxt.sync().await?;

    Ok(())
}

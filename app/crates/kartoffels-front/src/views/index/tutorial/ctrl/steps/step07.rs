use super::prelude::*;

static MSG: LazyLock<Msg> = LazyLock::new(|| Msg {
    title: Some(" tutorial (7/16) "),

    body: vec![
        MsgLine::new(
            "anyway, close this message to resume the game and let's see the \
             bot in action",
        ),
        MsgLine::new(""),
        MsgLine::new(
            "if everything goes correctly, we should see the machine driving \
             in squares, *how exquisite*!",
        ),
    ],

    buttons: vec![MsgButton::confirm("next", ())],
});

pub async fn run(ctxt: &mut TutorialCtxt) -> Result<()> {
    ctxt.game.msg(&MSG).await?;
    ctxt.game.resume().await?;

    time::sleep(Duration::from_secs(6)).await;

    ctxt.delete_bots().await?;
    ctxt.sync().await?;

    Ok(())
}

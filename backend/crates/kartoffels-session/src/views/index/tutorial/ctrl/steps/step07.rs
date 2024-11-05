use super::prelude::*;

static MSG: LazyLock<Msg> = LazyLock::new(|| Msg {
    title: Some(" tutorial (7/16) "),

    body: vec![
        MsgLine::new(
            "anyway, close this message to resume the game and let's see the \
             robot in action",
        ),
        MsgLine::new(""),
        MsgLine::new(
            "if everything goes correctly, we should see the robot driving in \
             squares, *how exquisite*!",
        ),
    ],

    buttons: vec![MsgButton::confirm(
        "yes, let's see the robot driving in squares",
        (),
    )],
});

pub async fn run(ctxt: &mut TutorialCtxt) -> Result<()> {
    ctxt.game.show_msg(&MSG).await?;
    ctxt.game.resume().await?;

    time::sleep(Duration::from_secs(6)).await;

    ctxt.destroy_bots().await?;
    ctxt.wait_for_ui().await?;

    Ok(())
}

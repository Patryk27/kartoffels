use super::prelude::*;

#[rustfmt::skip]
static DIALOG: LazyLock<Dialog<()>> = LazyLock::new(|| Dialog {
    title: Some(" tutorial "),

    body: vec![
        DialogLine::new(
            "the game is currently paused - close this window, press [`spc`] \
             (aka space) to resume and let's see the bot in action",
        ),
        DialogLine::new(""),
        DialogLine::new(
            "if everything goes correctly, we should see the robot driving in \
             squares, *how exquisite*!",
        ),
    ],

    buttons: vec![
        DialogButton::confirm("yes, let's see the robot driving in squares", ()),
    ],
});

#[rustfmt::skip]
static HELP: LazyLock<HelpDialog> = LazyLock::new(|| Dialog {
    title: Some(" help "),

    body: vec![
        DialogLine::new("press space to resume the game"),
    ],

    buttons: vec![DialogButton::confirm("got it", HelpDialogResponse::Close)],
});

pub async fn run(ctxt: &mut StepCtxt<'_>) -> Result<()> {
    ctxt.dialog(&DIALOG).await?;
    ctxt.game.set_help(&HELP).await?;

    ctxt.game
        .update_policy(|policy| {
            policy.user_can_pause_world = true;
        })
        .await?;

    ctxt.game
        .poll(|ctxt| {
            if ctxt.paused {
                Poll::Pending
            } else {
                Poll::Ready(())
            }
        })
        .await?;

    time::sleep(Duration::from_secs(6)).await;

    Ok(())
}

use super::prelude::*;

static DIALOG: LazyLock<Dialog<()>> = LazyLock::new(|| Dialog {
    title: Some(" tutorial "),

    body: vec![
        DialogLine::new("yes... ha ha ha... *YES*!"),
        DialogLine::new(""),
        DialogLine::new(
            "by telling the robot to always move forward instead of driving in \
             squares, we should see the robot, well, moving forward and \
             unknowingly falling out of the map",
        ),
    ],

    buttons: vec![DialogButton::confirm("let the hunger games begin", ())],
});

static DIALOG_RETRY: LazyLock<Dialog<()>> = LazyLock::new(|| Dialog {
    title: Some(" tutorial "),

    body: vec![
        DialogLine::new("hmm, your robot seems to be still alive"),
        DialogLine::new(""),
        DialogLine::new(
            "this wasn't a triumph, i'm making a note here, NOT a huge \
             success",
        ),
        DialogLine::new(""),
        DialogLine::new(
            "make sure you've removed the call to `motor_turn_right()` and \
             upload the bot again",
        ),
    ],

    buttons: vec![DialogButton::confirm("let's try again", ())],
});

static HELP_RETRY: LazyLock<HelpDialog> = LazyLock::new(|| Dialog {
    title: Some(" help "),

    body: vec![
        DialogLine::new(
            "make sure you've removed the call to `motor_turn_right()` and \
             upload the bot again",
        ),
        DialogLine::web(""),
        DialogLine::web("!! don't forget to re-run `./build` !!"),
    ],

    buttons: vec![HelpDialogResponse::close()],
});

pub async fn run(ctxt: &mut StepCtxt) -> Result<()> {
    ctxt.game.run_dialog(&DIALOG).await?;
    ctxt.game.resume().await?;

    loop {
        ctxt.game.set_status(Some("WATCHING".into())).await?;

        let result = time::timeout(
            Duration::from_secs(10),
            ctxt.snapshots.wait_until_bot_is_killed(),
        )
        .await;

        ctxt.destroy_bots().await?;
        ctxt.game.set_status(None).await?;

        match result {
            Ok(result) => {
                return result;
            }

            Err(_) => {
                ctxt.game.run_dialog(&DIALOG_RETRY).await?;
                ctxt.game.set_help(Some(&HELP_RETRY)).await?;
                ctxt.snapshots.wait_until_bot_is_spawned().await?;
                ctxt.game.set_help(None).await?;
            }
        }
    }
}

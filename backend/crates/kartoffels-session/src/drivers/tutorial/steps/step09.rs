use super::prelude::*;

#[rustfmt::skip]
static DIALOG: LazyLock<Dialog<()>> = LazyLock::new(|| Dialog {
    title: Some(" tutorial "),

    body: vec![
        DialogLine::new("*time for some fun!*"),
        DialogLine::new(""),
    ]
    .into_iter()
    .chain(INSTRUCTION.clone())
    .collect(),

    buttons: vec![
        DialogButton::confirm("i'm ready sir", ()),
    ],
});

#[rustfmt::skip]
static HELP: LazyLock<HelpDialog> = LazyLock::new(|| Dialog {
    title: Some(" help "),
    body: INSTRUCTION.clone(),

    buttons: vec![
        DialogButton::confirm("got it", HelpDialogResponse::Close),
    ],
});

#[rustfmt::skip]
static INSTRUCTION: LazyLock<Vec<DialogLine>> = LazyLock::new(|| vec![
    DialogLine::new(
        "remove the call to `motor_turn_right()`, so that everything the robot \
         does is just `motor_wait()` and `motor_step()`, then close this \
         message and upload the updated bot",
    ),
    DialogLine::web(""),
    DialogLine::web("!! don't forget to re-run `./build` !!"),
]);

pub async fn run(ctxt: &mut StepCtxt) -> Result<()> {
    ctxt.game.run_dialog(&DIALOG).await?;
    ctxt.game.set_help(Some(&HELP)).await?;
    ctxt.wait_until_bot_is_spawned().await?;
    ctxt.game.set_help(None).await?;
    ctxt.game.pause().await?;

    Ok(())
}

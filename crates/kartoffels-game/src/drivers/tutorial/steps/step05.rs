use super::prelude::*;

#[rustfmt::skip]
static DIALOG: LazyLock<Dialog<()>> = LazyLock::new(|| Dialog {
    title: Some(" tutorial "),

    body: vec![
        DialogLine::new(
            "as you can see, the code in `main.rs` just calls a couple of \
             functions in a loop - but before we jump into more thorough \
             explanations, let's see the robot in action"
        ),
        DialogLine::new(""),
    ]
    .into_iter()
    .chain(INSTRUCTION.clone())
    .collect(),

    buttons: vec![DialogButton::confirm("i have done so", ())],
});

#[rustfmt::skip]
static HELP: LazyLock<HelpDialog> = LazyLock::new(|| Dialog {
    title: Some(" help "),
    body: INSTRUCTION.clone(),
    buttons: vec![DialogButton::confirm("got it", HelpDialogResponse::Close)],
});

#[rustfmt::skip]
static INSTRUCTION: LazyLock<Vec<DialogLine>> = LazyLock::new(|| vec![
    DialogLine::new("if you're on linux, macos, freebsd etc., run this:"),
    DialogLine::web("    ./build"),
    DialogLine::ssh("    ./build --copy"),
    DialogLine::new(""),
    DialogLine::new("if you're on windows, run this:"),
    DialogLine::web("    ./build.bat"),
    DialogLine::ssh("    ./build.bat --copy"),
    DialogLine::new(""),
    DialogLine::new(
        "having done so, press enter to close this window and then press [`u`] \
         to upload the bot",
    ),
    DialogLine::web(""),
    DialogLine::web(
        "when the file picker opens, choose a file called `kartoffel` - it \
         should be located next to `README.md` etc.",
    ),
]);

pub async fn run(ctxt: &mut StepCtxt<'_>) -> Result<()> {
    ctxt.dialog(&DIALOG).await?;
    ctxt.game.set_help(&HELP).await?;

    ctxt.game
        .update_policy(|policy| {
            policy.ui_enabled = true;
        })
        .await?;

    ctxt.game
        .poll(|ctxt| {
            if ctxt.world.bots().alive().is_empty() {
                Poll::Pending
            } else {
                Poll::Ready(())
            }
        })
        .await?;

    ctxt.game.pause().await?;

    Ok(())
}

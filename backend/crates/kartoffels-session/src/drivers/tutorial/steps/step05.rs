use super::prelude::*;

static DIALOG: LazyLock<Dialog<()>> = LazyLock::new(|| Dialog {
    title: Some(" tutorial "),

    body: vec![
        DialogLine::new(
            "as you can see, the code in `main.rs` just calls a couple of \
             functions in a loop - but before we jump into explanations, let's \
             see the robot in action!",
        ),
        DialogLine::new(""),
    ]
    .into_iter()
    .chain(INSTRUCTION.clone())
    .collect(),

    buttons: vec![DialogButton::confirm("i have done so", ())],
});

static HELP: LazyLock<HelpDialog> = LazyLock::new(|| Dialog {
    title: Some(" help "),
    body: INSTRUCTION.clone(),
    buttons: vec![HelpDialogResponse::close()],
});

static INSTRUCTION: LazyLock<Vec<DialogLine>> = LazyLock::new(|| {
    vec![
        DialogLine::new("if you're on linux, macos, freebsd etc., run this:"),
        DialogLine::web("    ./build"),
        DialogLine::ssh("    ./build --copy"),
        DialogLine::new(""),
        DialogLine::new("if you're on windows, run this:"),
        DialogLine::web("    ./build.bat"),
        DialogLine::ssh("    ./build.bat --copy"),
        DialogLine::new(""),
        DialogLine::new(
            "... and having done so, press [`enter`] to close this message and \
             then press [`u`] to upload the bot",
        ),
        DialogLine::web(""),
        DialogLine::web(
            "when the file picker opens, choose a file called `kartoffel` - it \
             should be located next to `README.md` etc.",
        ),
    ]
});

pub async fn run(ctxt: &mut StepCtxt) -> Result<()> {
    ctxt.game.run_dialog(&DIALOG).await?;
    ctxt.game.set_help(Some(&HELP)).await?;
    ctxt.wait_until_bot_is_spawned().await?;
    ctxt.game.set_help(None).await?;
    ctxt.game.pause().await?;

    Ok(())
}

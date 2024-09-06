use super::prelude::*;

#[rustfmt::skip]
static DIALOG: LazyLock<Dialog<()>> = LazyLock::new(|| Dialog {
    title: Some(" tutorial "),

    body: vec![
        DialogLine::new("lesson #1:").bold(),
        DialogLine::new("- you can navigate the interface using keyboard and/or mouse"),
        DialogLine::ssh("  (that includes when you're connected through the terminal)"),
        DialogLine::new(""),
        DialogLine::new("lesson #2:").bold(),
        DialogLine::new("- pressing Ctrl-a will bring you to the main menu"),
        DialogLine::ssh("- pressing Ctrl-c will disconnect you"),
        DialogLine::new(""),
        DialogLine::new("lesson #3:").bold(),
        DialogLine::new("- don't lick yellow snow"),
    ],

    buttons: vec![
        DialogButton::confirm("sure", ()),
    ],
});

#[allow(clippy::while_let_loop)]
pub async fn run(ctxt: &mut StepCtxt<'_>) -> Result<()> {
    ctxt.dialog(&DIALOG).await?;

    Ok(())
}

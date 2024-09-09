use super::prelude::*;

#[rustfmt::skip]
static DIALOG: LazyLock<Dialog<()>> = LazyLock::new(|| Dialog {
    title: Some(" tutorial "),

    body: vec![
        DialogLine::new("nice!"),
        DialogLine::new(""),
        DialogLine::new("i mean, not nice, because we're dead, but baby steps"),
        DialogLine::new(""),
        DialogLine::new(
            "now it's time for you to learn about *da radar*, using which you \
             can program the bot to avoid falling out the map",
        ),
    ],

    buttons: vec![
        DialogButton::confirm("let's learn", ()),
    ],
});

pub async fn run(ctxt: &mut StepCtxt) -> Result<()> {
    ctxt.run_dialog(&DIALOG).await?;

    Ok(())
}

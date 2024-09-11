use super::prelude::*;

#[rustfmt::skip]
static DIALOG: LazyLock<Dialog<()>> = LazyLock::new(|| Dialog {
    title: Some(" tutorial "),

    body: vec![
        DialogLine::new("nice!"),
        DialogLine::new(""),
        DialogLine::new(
            "you, _[subject name here]_ must be the pride of _[subject \
             hometown here]_",
        )
    ],

    buttons: vec![
        DialogButton::confirm("i am", ()),
    ],
});

pub async fn run(ctxt: &mut StepCtxt) -> Result<()> {
    ctxt.run_dialog(&DIALOG).await?;

    Ok(())
}

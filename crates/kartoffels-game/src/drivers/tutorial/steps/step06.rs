use super::prelude::*;

#[rustfmt::skip]
static DIALOG: LazyLock<Dialog<'static, ()>> = LazyLock::new(|| Dialog {
    title: Some(" tutorial "),

    body: vec![
        DialogLine::raw("nice!"),
        DialogLine::raw(""),
        DialogLine::raw(
            "you, [subject name here] must be the pride of [subject hometown \
             here].",
        )
    ],

    buttons: vec![
        DialogButton::confirm("i am", ()),
    ],
});

pub async fn run(ctxt: &mut StepCtxt<'_>) -> Result<()> {
    ctxt.dialog(&DIALOG).await?;

    Ok(())
}

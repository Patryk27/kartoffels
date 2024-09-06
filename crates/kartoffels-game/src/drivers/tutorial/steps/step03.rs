use super::prelude::*;

#[rustfmt::skip]
static DIALOG: LazyLock<Dialog<'static, ()>> = LazyLock::new(|| Dialog {
    title: Some(" tutorial "),

    body: vec![
        DialogLine::raw(
            "perhaps i should mention that we'll be using rust, are you \
             comfortable with that?",
        ),
    ],

    buttons: vec![
        DialogButton::confirm("sure why not", ()),
    ],
});

pub async fn run(ctxt: &mut StepCtxt<'_>) -> Result<()> {
    ctxt.dialog(&DIALOG).await?;

    Ok(())
}

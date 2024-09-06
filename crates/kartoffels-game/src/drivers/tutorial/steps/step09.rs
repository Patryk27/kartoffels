use super::prelude::*;

#[rustfmt::skip]
static DIALOG: LazyLock<Dialog<()>> = LazyLock::new(|| Dialog {
    title: Some(" tutorial "),

    body: vec![
        DialogLine::new(
            "now, comment out the call to `motor_turn_right()`, so that \
             everything the robot does is just `motor_wait()` and \
             `motor_step()`"
        ),
        DialogLine::new(""),
        DialogLine::new(
            "having done so, close this window and upload the new bot",
        ),
        DialogLine::web(""),
        DialogLine::web("!! also, don't forget to re-run `./build` !!"),
    ],

    buttons: vec![
        DialogButton::confirm("done", ()),
    ],
});

pub async fn run(ctxt: &mut StepCtxt<'_>) -> Result<()> {
    ctxt.dialog(&DIALOG).await?;

    Ok(())
}

use super::prelude::*;

#[rustfmt::skip]
static DIALOG: LazyLock<Dialog<'static, ()>> = LazyLock::new(|| Dialog {
    title: Some(" tutorial "),

    body: vec![
        DialogLine::raw("faboulous!").fg(theme::PINK).bold(),
        DialogLine::raw(""),
        DialogLine::raw(
            "launch vscode, vim, emacs or whatever makes your life colorful \
             and open `main.rs` from the cloned repository"
        ),
        DialogLine::raw(""),
        DialogLine::raw(
            "for, you see, writing a bot is similar to writing a regular rust \
             program - but it's also different, mucho different",
        ),
        DialogLine::raw(""),
        DialogLine::raw(
            "say, you don't have access to the standard library - there's no \
             `println!()`, no `std::fs` etc; everything your robot has access \
             to is a bit of memory, motor, radar and serial port, like the \
             people in ancient rome did",
        ),
    ],

    buttons: vec![
        DialogButton::confirm("got it", ()),
    ],
});

pub async fn run(ctxt: &mut StepCtxt<'_>) -> Result<()> {
    ctxt.dialog(&DIALOG).await?;

    Ok(())
}

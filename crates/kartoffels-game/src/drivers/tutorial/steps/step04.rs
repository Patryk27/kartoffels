use super::prelude::*;

#[rustfmt::skip]
static DIALOG: LazyLock<Dialog<()>> = LazyLock::new(|| Dialog {
    title: Some(" tutorial "),

    body: vec![
        DialogLine::new("faboulous!").fg(theme::PINK).bold(),
        DialogLine::new(""),
        DialogLine::new(
            "launch vscode, vim, emacs or whatever makes your life colorful \
             and open `main.rs` from the cloned repository"
        ),
        DialogLine::new(""),
        DialogLine::new(
            "for, you see, writing a bot is similar to writing a regular rust \
             program - but it's also different, mucho different",
        ),
        DialogLine::new(""),
        DialogLine::new(
            "like, _you-dont-have-access-to-standard-library_ different"
        ),
        DialogLine::new(""),
        DialogLine::new(
            "there's no `println!()`, no `std::fs` etc., everything a bot has \
             access to is a bit of memory, motor, radar and serial port",
        ),
        DialogLine::new(""),
        DialogLine::new("you know, like the people in ancient rome did"),
    ],

    buttons: vec![
        DialogButton::confirm("got it", ()),
    ],
});

pub async fn run(ctxt: &mut StepCtxt<'_>) -> Result<()> {
    ctxt.dialog(&DIALOG).await?;

    Ok(())
}

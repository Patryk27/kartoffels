use super::prelude::*;

static DIALOG: LazyLock<Dialog<()>> = LazyLock::new(|| Dialog {
    title: Some(" tutorial (4/16) "),

    body: vec![
        DialogLine::new("fabulous!").fg(theme::PINK).bold(),
        DialogLine::new(""),
        DialogLine::new(
            "now launch vscode, vim, emacs or whatever gives your life colors \
             and open `main.rs` from the cloned repository",
        ),
        DialogLine::new(""),
        DialogLine::new(
            "for, you see, writing a bot is similar to writing a regular rust \
             program - but it's also different, _mucho_ different",
        ),
        DialogLine::new(""),
        DialogLine::new(
            "like, you-dont-have-access-to-standard-library different",
        ),
        DialogLine::new(""),
        DialogLine::new(
            "this means there's no `println!()`, no `std::fs`, everything the \
             robot has access to is a bit of memory, motor, radar and serial \
             port",
        ),
        DialogLine::new(""),
        DialogLine::new("you know, like people in ancient rome did"),
    ],

    buttons: vec![DialogButton::confirm("got it", ())],
});

pub async fn run(ctxt: &mut StepCtxt) -> Result<()> {
    ctxt.game.run_dialog(&DIALOG).await?;

    Ok(())
}

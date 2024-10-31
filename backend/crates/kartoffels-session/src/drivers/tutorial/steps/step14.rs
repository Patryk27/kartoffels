use super::prelude::*;

static DIALOG: LazyLock<Dialog<()>> = LazyLock::new(|| Dialog {
    title: Some(" tutorial "),

    body: vec![
        DialogLine::new("congrats!"),
        DialogLine::new(""),
        DialogLine::new(
            "i don't want to keep you for much longer, so let's wrap things up \
             with a lesson on the last peripheral you need to know in order to \
             play:",
        ),
        DialogLine::new(""),
        DialogLine::new("🔪 the knife 🔪").centered().fg(theme::YELLOW).bold(),
    ],

    buttons: vec![DialogButton::confirm("let's take a stab at it", ())],
});

pub async fn run(ctxt: &mut StepCtxt) -> Result<()> {
    ctxt.destroy_bots().await?;
    ctxt.wait_for_ui().await?;
    ctxt.game.run_dialog(&DIALOG).await?;

    Ok(())
}

use super::prelude::*;

static DIALOG: LazyLock<Dialog<()>> = LazyLock::new(|| Dialog {
    title: Some(" tutorial "),

    body: vec![
        DialogLine::new("✨ *you made it!* ✨")
            .fg(theme::GREEN)
            .bold()
            .centered(),
        DialogLine::new(""),
        DialogLine::new(
            "your progress over the previous couple of minutes was incredible \
             - you can now go and conquer the world, legally!",
        )
        .centered(),
        DialogLine::new(""),
        DialogLine::new("🥔 have fun and remember the power of potato 🥔")
            .centered(),
    ],

    buttons: vec![DialogButton::confirm("thanks m8", ())],
});

pub async fn run(ctxt: &mut StepCtxt) -> Result<()> {
    ctxt.world.set_map(Default::default()).await?;
    ctxt.wait_for_ui().await?;
    ctxt.game.run_dialog(&DIALOG).await?;

    Ok(())
}

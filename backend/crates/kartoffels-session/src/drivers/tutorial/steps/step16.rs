use super::prelude::*;

static DIALOG: LazyLock<Dialog<()>> = LazyLock::new(|| Dialog {
    title: Some(" tutorial (16/16) "),

    body: vec![
        DialogLine::new("✨ *yay, you made it!* ✨")
            .fg(theme::GREEN)
            .bold()
            .centered(),
        DialogLine::new(""),
        DialogLine::new(
            "not sure if your mom and dad are proud of you, but _i_ am !!",
        )
        .centered(),
        DialogLine::new(""),
        DialogLine::new("now, waste no time:").centered(),
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

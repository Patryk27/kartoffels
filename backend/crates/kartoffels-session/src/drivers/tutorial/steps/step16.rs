use super::prelude::*;

#[rustfmt::skip]
static DIALOG: LazyLock<Dialog<()>> = LazyLock::new(|| Dialog {
    title: Some(" tutorial "),

    body: vec![
        DialogLine::new("✨ *you made it!* ✨").fg(theme::GREEN).bold().centered(),
        DialogLine::new(""),
        DialogLine::new(
            "look at you, sailing through the air majestically... like an \
             eagle... piloting a blimp",
        ).centered(),
        DialogLine::new(""),
        DialogLine::new(
            "your progress over the previous couple of minutes was incredible \
             - you can now go and conquer the world, legally!",
        ).centered(),
        DialogLine::new(""),
        DialogLine::new("🥔 have fun and remember the power of potato 🥔").centered(),
    ],

    buttons: vec![
        DialogButton::confirm("thanks m8", ()),
    ],
});

pub async fn run(ctxt: &mut StepCtxt) -> Result<()> {
    ctxt.game.run_dialog(&DIALOG).await?;

    Ok(())
}

use super::prelude::*;

#[rustfmt::skip]
static DIALOG: LazyLock<Dialog<'static, ()>> = LazyLock::new(|| Dialog {
    title: Some(" tutorial "),

    body: vec![
        DialogLine::raw("cool!").fg(theme::GREEN),
        DialogLine::raw(""),
        DialogLine::raw(
            "i mean, not cool, because we're dead - but relatively speaking \
             it's progress",
        ),
        DialogLine::raw(""),
        DialogLine::raw(
            "after a bot dies, the game automatically revives it - close this \
             dialog, unpause the game and let's see that",
        ),
    ],

    buttons: vec![
        DialogButton::confirm("let's see the robot dying again", ()),
    ],
});

#[rustfmt::skip]
static HELP: LazyLock<HelpDialog> = LazyLock::new(|| Dialog {
    title: Some(" help "),

    body: vec![
        DialogLine::raw(
            "press space to unpause the game and let's see the bot in action, \
             again",
        ),
    ],

    buttons: vec![DialogButton::confirm("got it", HelpDialogResponse::Close)],
});

pub async fn run(ctxt: &mut StepCtxt<'_>) -> Result<()> {
    ctxt.game.pause().await?;

    ctxt.dialog(&DIALOG).await?;
    ctxt.game.set_help(&HELP).await?;

    let mut events = ctxt.world().events();

    loop {
        let event = events.next().await.context("world has crashed")?;

        if let Event::BotKilled { .. } = &*event {
            return Ok(());
        }
    }
}

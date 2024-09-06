use super::prelude::*;

#[rustfmt::skip]
static DIALOG: LazyLock<Dialog<'static, ()>> = LazyLock::new(|| Dialog {
    title: Some(" tutorial "),

    body: vec![
        DialogLine::raw(
            "the game has been automatically paused to show you the humoristic \
             element a moment ago",
        ),
        DialogLine::raw(""),
        DialogLine::raw(
            "now, close this dialogue, press space to unpause the game and \
             let's see the bot in action",
        ),
        DialogLine::raw(""),
        DialogLine::raw(
            "if everything goes correctly, we should see the robot driving \
             forward and falling out the map",
        ),
    ],

    buttons: vec![
        DialogButton::confirm("let's see the robot driving", ()),
    ],
});

#[rustfmt::skip]
static HELP: LazyLock<HelpDialog> = LazyLock::new(|| Dialog {
    title: Some(" help "),

    body: vec![
        DialogLine::raw(
            "press space to unpause the game and let's see the bot in action",
        ),
    ],

    buttons: vec![DialogButton::confirm("got it", HelpDialogResponse::Close)],
});

pub async fn run(ctxt: &mut StepCtxt<'_>) -> Result<()> {
    ctxt.dialog(&DIALOG).await?;
    ctxt.game.set_help(&HELP).await?;

    ctxt.game
        .update_policy(|policy| {
            policy.user_can_pause_world = true;
        })
        .await?;

    let mut events = ctxt.world().events();

    loop {
        let event = events.next().await?;

        if let Event::BotKilled { .. } = &*event {
            return Ok(());
        }
    }
}

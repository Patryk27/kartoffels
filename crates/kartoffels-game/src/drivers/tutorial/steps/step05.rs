use super::prelude::*;

#[rustfmt::skip]
static INSTRUCTION: LazyLock<Vec<DialogLine<'static>>> = LazyLock::new(|| vec![
    DialogLine::raw("if you're on windows, run this:"),
    DialogLine::web("    ./build.bat").fg(theme::WASHED_PINK),
    DialogLine::ssh("    ./build.bat --copy").fg(theme::WASHED_PINK),
    DialogLine::raw(""),
    DialogLine::raw("otherwise, run this:"),
    DialogLine::web("    ./build").fg(theme::WASHED_PINK),
    DialogLine::ssh("    ./build --copy").fg(theme::WASHED_PINK),
    DialogLine::raw(""),
    DialogLine::raw(
        "... and having done so, press enter to close this window and then \
         press `u` to upload the bot",
    ),
    DialogLine::web(""),
    DialogLine::web(
        "when the file picker opens, choose a file called `kartoffel` - it \
         should be located next to `README.md` etc.",
    ),
]);

#[rustfmt::skip]
static DIALOG: LazyLock<Dialog<'static, ()>> = LazyLock::new(|| Dialog {
    title: Some(" tutorial "),

    body: vec![
        DialogLine::raw(
            "anyway, as you can see in the code, our robot currently doesn't \
             do much - it just calls `motor_step()` over and over",
        ),
        DialogLine::raw(""),
        DialogLine::raw(
            "this function is responsible for moving the robot one tile \
             forward in the direction it is currently facing",
        ),
        DialogLine::raw(""),
        DialogLine::from_iter([
            Span::raw("boooring").bold(),
            Span::raw(" - let's see the robot in action !!"),
        ]),
        DialogLine::raw(""),
    ]
    .into_iter()
    .chain(INSTRUCTION.clone())
    .collect(),

    buttons: vec![DialogButton::confirm("i have done so", ())],
});

#[rustfmt::skip]
static HELP: LazyLock<HelpDialog> = LazyLock::new(|| Dialog {
    title: Some(" help "),
    body: INSTRUCTION.clone(),
    buttons: vec![DialogButton::confirm("got it", HelpDialogResponse::Close)],
});

pub async fn run(ctxt: &mut StepCtxt<'_>) -> Result<()> {
    ctxt.dialog(&DIALOG).await?;
    ctxt.game.set_help(&HELP).await?;

    ctxt.world = Some(ctxt.store.create_world(Config {
        name: "sandbox".into(),
        mode: ModeConfig::Deathmatch(DeathmatchModeConfig {
            round_duration: None,
        }),
        theme: ThemeConfig::Arena(ArenaThemeConfig { radius: 12 }),
        policy: WorldPolicy {
            max_alive_bots: 16,
            max_queued_bots: 16,
        },
    }));

    let mut events = ctxt.world().events();

    ctxt.game.join(ctxt.world().clone()).await?;

    ctxt.game
        .update_policy(|policy| {
            policy.ui_enabled = true;
        })
        .await?;

    ctxt.game
        .poll(|world| {
            if world.bots().alive().is_empty() {
                Poll::Pending
            } else {
                Poll::Ready(())
            }
        })
        .await?;

    ctxt.game.pause().await?;

    loop {
        if let Event::BotSpawned { .. } = &*events.next().await? {
            return Ok(());
        }
    }
}

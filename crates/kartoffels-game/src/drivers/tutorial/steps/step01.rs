use super::prelude::*;

#[rustfmt::skip]
static DIALOG: LazyLock<Dialog<bool>> = LazyLock::new(|| Dialog {
    title: Some(" tutorial "),

    body: vec![
        DialogLine::new("hey there and welcome to kartoffels 🫡"),
        DialogLine::new(""),
        DialogLine::from_iter([
            Span::raw(
                "in just a couple of minutes we're going to make a bots' boss \
                 out of you, so buckle up and let's get started! "
            ),
            Span::raw("*").fg(theme::RED),
        ]),
        DialogLine::new(""),
        DialogLine::new("ready?").fg(theme::GREEN).bold().centered(),
        DialogLine::new(""),
        DialogLine::from_iter([
            Span::raw("* ").fg(theme::RED),
            Span::raw(
                "kartoffels ltd is not responsible for loss of hearing, loss \
                 of sight, sudden feeling of the flight and fight syndrome, \
                 wanting to do origami but being unable to etc."
            ),
        ]).fg(theme::DARK_GRAY).right_aligned(),
    ],

    buttons: vec![
        DialogButton::abort("no, leave tutorial", false),
        DialogButton::confirm("yes, start tutorial", true),
    ],
});

pub async fn run(ctxt: &mut StepCtxt<'_>) -> Result<bool> {
    ctxt.game
        .set_policy(Policy {
            ui_enabled: false,
            user_can_pause_world: false,
            user_can_configure_world: false,
            user_can_manage_bots: false,
            pause_is_propagated: true,
        })
        .await?;

    ctxt.world = Some({
        let world = ctxt.store.create_world(Config {
            name: "sandbox".into(),
            mode: ModeConfig::Deathmatch(DeathmatchModeConfig {
                round_duration: None,
            }),
            theme: ThemeConfig::Arena(ArenaThemeConfig { radius: 12 }),
            policy: WorldPolicy {
                max_alive_bots: 16,
                max_queued_bots: 16,
            },
        });

        world.set_spawn_point(ivec2(12, 12)).await?;
        world
    });

    ctxt.game.join(ctxt.world().clone()).await?;
    ctxt.dialog(&DIALOG).await
}

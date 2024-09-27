use super::prelude::*;
use super::Challenge;

pub static CHALLENGE: Challenge = Challenge {
    name: "lost in a labirynth",
    desc: "your bot got lost in a labirynth and it's very distressed - help it escape",
    run,
};

#[rustfmt::skip]
static DIALOG: LazyLock<Dialog<bool>> = LazyLock::new(|| Dialog {
    title: Some(" get me out of here "),

    body: vec![
        DialogLine::new(
            "your bot got lost in a mythical labirynth and it's very \
             distressed - implement a firmware to help your bot escape",
        ),
    ],

    buttons: vec![
        DialogButton::abort("go back", false),
        DialogButton::confirm("let's do it", true),
    ],
});

fn run(store: &Store, game: DrivenGame) -> BoxFuture<'_, Result<()>> {
    Box::pin(async move {
        game.set_perms(Permissions::CHALLENGE).await?;

        let _world = store.create_world(Config {
            clock: Default::default(),
            mode: Mode::Deathmatch(DeathmatchMode::default()),
            name: "sandbox".into(),
            path: Default::default(),
            policy: Policy {
                auto_respawn: true,
                max_alive_bots: 1,
                max_queued_bots: 1,
            },
            rng: None,
            theme: None,
        })?;

        if !game.run_dialog(&*DIALOG).await? {
            return Ok(());
        }

        Ok(())
    })
}

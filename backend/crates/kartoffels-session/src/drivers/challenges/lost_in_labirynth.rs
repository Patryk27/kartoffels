use super::prelude::*;

pub static CHALLENGE: Challenge = Challenge {
    name: "lost in labirynth",
    desc: "your bot got lost and it's visibly distressed - help it escape!",
    run,
};

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

        future::pending::<()>().await;

        Ok(())
    })
}

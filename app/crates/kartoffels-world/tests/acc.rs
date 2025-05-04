mod acc {
    use super::*;

    mod bots;
    mod meta;
}

use glam::{ivec2, uvec2};
use indoc::indoc;
use kartoffels_prefabs::*;
use kartoffels_utils::*;
use kartoffels_world::prelude::*;
use pretty_assertions as pa;
use std::future::Future;
use std::path::Path;
use std::sync::Arc;

#[tokio::test]
async fn smoke() {
    let world = kartoffels_world::create(config());
    let mut asserter = asserter("smoke");

    for _ in 0..16 {
        world
            .create_bot(CreateBotRequest::new(ROBERTO))
            .await
            .unwrap();
    }

    world.assert(&mut asserter, "1.md").await;
    world.assert_json(&mut asserter, "1.json").await;

    world.tick(256_000).await.unwrap();

    world.assert(&mut asserter, "2.md").await;
    world.assert_json(&mut asserter, "2.json").await;
}

fn config() -> Config {
    Config {
        clock: Clock::manual(),
        events: false,
        name: "world".into(),
        policy: Policy {
            auto_respawn: true,
            max_alive_bots: 16,
            max_queued_bots: 16,
        },
        seed: Some(Default::default()),
        theme: Some(Theme::Arena(ArenaTheme::new(12))),
    }
}

fn asserter(test: &str) -> Asserter {
    Asserter::new(Path::new("tests").join("acc").join(test))
}

trait HandleExt {
    fn snapshot(&self) -> impl Future<Output = Arc<Snapshot>>;

    fn assert(
        &self,
        asserter: &mut Asserter,
        file: &str,
    ) -> impl Future<Output = ()>;

    fn assert_json(
        &self,
        asserter: &mut Asserter,
        file: &str,
    ) -> impl Future<Output = ()>;
}

impl HandleExt for Handle {
    async fn snapshot(&self) -> Arc<Snapshot> {
        self.snapshots().next().await.unwrap()
    }

    async fn assert(&self, asserter: &mut Asserter, file: &str) {
        let actual = self.snapshot().await;
        let actual = format!("{}\n", actual.to_string().trim_end());

        asserter.assert(file, actual);
    }

    async fn assert_json(&self, asserter: &mut Asserter, file: &str) {
        let actual = self.snapshot().await;
        let actual = serde_json::to_string_pretty(&actual).unwrap();

        asserter.assert(file, actual);
    }
}

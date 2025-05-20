use anyhow::{Context, Result};
use clap::Parser;
use glam::uvec2;
use kartoffels_prefabs::ROBERTO;
use kartoffels_world::prelude::*;
use std::time::Duration;
use tokio::time;
use tracing::info;

#[derive(Debug, Parser)]
pub struct BenchCmd;

impl BenchCmd {
    pub(crate) fn run(self) -> Result<()> {
        tracing_subscriber::fmt()
            .with_env_filter("kartoffels=debug")
            .init();

        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?
            .block_on(async {
                info!("creating world");

                let world = kartoffels_world::create(Config {
                    clock: Clock::Unlimited,
                    events: false,
                    name: "bench".into(),
                    policy: Policy {
                        allow_breakpoints: false,
                        auto_respawn: true,
                        max_alive_bots: 128,
                        max_queued_bots: 256,
                    },
                    seed: Default::default(),
                    theme: Some(Theme::Cave(CaveTheme::new(uvec2(64, 32)))),
                });

                info!("creating bots");

                for _ in 0..128 {
                    world
                        .create_bot(CreateBotRequest::new(ROBERTO))
                        .await
                        .context("couldn't create bot")?;
                }

                info!("ready; benchmarking for 10s");

                time::sleep(Duration::from_secs(10)).await;

                info!("shutting down");

                world.shutdown().await?;

                Ok(())
            })
    }
}

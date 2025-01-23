use anyhow::{anyhow, Context, Result};
use clap::Parser;
use kartoffels_utils::Id;
use kartoffels_world::prelude::{Config, Policy, Theme};
use rand::Rng;
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub struct CreateWorldCmd {
    data: PathBuf,

    #[clap(long)]
    name: String,

    #[clap(long, default_value = "")]
    policy: String,

    #[clap(long)]
    theme: String,
}

impl CreateWorldCmd {
    pub(crate) fn run(self) -> Result<()> {
        if !self.data.exists() {
            return Err(anyhow!(
                "data directory doesn't exist: {}",
                self.data.display()
            ));
        }

        let id = rand::thread_rng().gen::<Id>();
        let path = self.data.join(format!("{id}.world"));
        let name = self.name;

        let policy =
            Policy::create(&self.policy).context("couldn't parse policy")?;

        let theme =
            Theme::create(&self.theme).context("couldn't parse theme")?;

        let config = Config {
            name,
            path: Some(path),
            policy,
            theme: Some(theme),
            ..Default::default()
        };

        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?
            .block_on(async move {
                kartoffels_world::create(config).shutdown().await?;

                println!("created world `{id}`");

                Ok(())
            })
    }
}

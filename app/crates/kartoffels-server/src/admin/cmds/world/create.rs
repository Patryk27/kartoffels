use super::CmdContext;
use anyhow::{Context, Result};
use clap::Parser;
use kartoffels_world::prelude::Config;
use std::fmt::Write;

#[derive(Debug, Parser)]
pub struct CreateWorldCmd {
    name: String,
    #[clap(long, default_value = "{}")]
    policy: String,
    #[clap(long)]
    theme: String,
}

impl CreateWorldCmd {
    pub async fn run(self, ctxt: &mut CmdContext<'_>) -> Result<()> {
        let policy = serde_json::from_str(&self.policy).with_context(|| {
            format!("couldn't parse policy: {}", self.policy)
        })?;

        let theme = serde_json::from_str(&self.theme)
            .with_context(|| format!("couldn't parse theme: {}", self.theme))?;

        let world = ctxt
            .store
            .create_public_world(Config {
                name: self.name,
                policy,
                theme: Some(theme),
                ..Default::default()
            })
            .await?;

        writeln!(ctxt, "ok, world {} created", world.id())?;

        Ok(())
    }
}

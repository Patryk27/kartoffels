use anyhow::{Context, Result};
use clap::Parser;
use kartoffels_store::Store;
use kartoffels_ui::Term;
use kartoffels_world::prelude::{Config, Policy, Theme};
use std::fmt::Write;
use std::str::FromStr;

#[derive(Debug, Parser)]
pub struct CreateWorldCmd {
    name: String,

    #[clap(long, default_value = "")]
    policy: String,

    #[clap(long)]
    theme: String,
}

impl CreateWorldCmd {
    pub(super) fn run(self, store: &Store, term: &mut Term) -> Result<()> {
        let policy = Policy::from_str(&self.policy).with_context(|| {
            format!("couldn't parse policy: {}", self.policy)
        })?;

        let theme = Theme::from_str(&self.theme)
            .with_context(|| format!("couldn't parse theme: {}", self.theme))?;

        let world = store.create_public_world(Config {
            name: self.name,
            policy,
            theme: Some(theme),
            ..Default::default()
        })?;

        writeln!(term, "id: {}", world.id())?;

        Ok(())
    }
}

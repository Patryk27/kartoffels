use crate::{ModeConfig, Policy, ThemeConfig};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub name: String,
    pub mode: ModeConfig,
    pub theme: ThemeConfig,
    pub policy: Policy,
}

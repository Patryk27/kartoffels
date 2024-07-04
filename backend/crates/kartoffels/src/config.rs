use crate::{ModeConfig, Policy, ThemeConfig, WorldName};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub name: WorldName,
    pub mode: ModeConfig,
    pub theme: ThemeConfig,
    pub policy: Policy,
}

use crate::{ModeConfig, ThemeConfig, WorldName};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct WorldConfig {
    pub name: WorldName,
    pub mode: ModeConfig,
    pub theme: ThemeConfig,
}

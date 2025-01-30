use anyhow::{anyhow, Error, Result};
use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct Secret(String);

impl Secret {
    pub const MAX_LENGTH: usize = 64;

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for Secret {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        if s.len() > Self::MAX_LENGTH {
            return Err(anyhow!(
                "secret is too long - the limit is {} characters",
                Self::MAX_LENGTH
            ));
        }

        if s.chars().any(|ch| ch.is_ascii_control()) {
            return Err(anyhow!("secret contains forbidden characters"));
        }

        Ok(Self(s.to_owned()))
    }
}

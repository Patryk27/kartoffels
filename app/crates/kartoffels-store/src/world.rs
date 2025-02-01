use anyhow::{anyhow, Error};
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorldType {
    Public,
    Private,
}

impl fmt::Display for WorldType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                WorldType::Public => "public",
                WorldType::Private => "private",
            }
        )
    }
}

impl FromStr for WorldType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "public" => Ok(Self::Public),
            "private" => Ok(Self::Private),
            _ => Err(anyhow!("unknown world type: {s}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display() {
        assert_eq!("public", WorldType::Public.to_string());
        assert_eq!("private", WorldType::Private.to_string());
    }

    #[test]
    fn from_str() {
        assert_eq!(WorldType::Public, WorldType::from_str("public").unwrap());
        assert_eq!(WorldType::Private, WorldType::from_str("private").unwrap());

        assert_eq!(
            "unknown world type: invalid",
            WorldType::from_str("invalid").unwrap_err().to_string()
        );
    }
}

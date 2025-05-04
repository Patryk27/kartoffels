use anyhow::{anyhow, Error};
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorldVis {
    Public,
    Private,
}

impl fmt::Display for WorldVis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Public => "public",
                Self::Private => "private",
            }
        )
    }
}

impl FromStr for WorldVis {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "public" => Ok(Self::Public),
            "private" => Ok(Self::Private),
            _ => Err(anyhow!("unknown world visibility: {s}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display() {
        assert_eq!("public", WorldVis::Public.to_string());
        assert_eq!("private", WorldVis::Private.to_string());
    }

    #[test]
    fn from_str() {
        assert_eq!(WorldVis::Public, WorldVis::from_str("public").unwrap());
        assert_eq!(WorldVis::Private, WorldVis::from_str("private").unwrap());

        assert_eq!(
            "unknown world visibility: invalid",
            WorldVis::from_str("invalid").unwrap_err().to_string()
        );
    }
}

use anyhow::{anyhow, Context, Error};
use itertools::Itertools;
use rand::distributions::Standard;
use rand::prelude::Distribution;
use rand::Rng;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::num::NonZeroU64;
use std::str::FromStr;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id(NonZeroU64);

impl Id {
    pub const LENGTH: usize = 19;

    pub const fn new(id: u64) -> Self {
        Self(NonZeroU64::new(id).unwrap())
    }

    pub fn try_new(id: u64) -> Option<Self> {
        NonZeroU64::new(id).map(Self)
    }

    pub fn get(self) -> u64 {
        self.0.into()
    }
}

impl Distribution<Id> for Standard {
    fn sample<R>(&self, rng: &mut R) -> Id
    where
        R: Rng + ?Sized,
    {
        Id(rng.gen())
    }
}

impl FromStr for Id {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((ab, cd, ef, gh)) = s.split('-').collect_tuple() else {
            return Err(anyhow!("invalid id format"));
        };

        let parse_part = |xy| {
            u16::from_str_radix(xy, 16)
                .context("invalid id format")
                .map(|xy| xy.to_be_bytes())
        };

        let [a, b] = parse_part(ab)?;
        let [c, d] = parse_part(cd)?;
        let [e, f] = parse_part(ef)?;
        let [g, h] = parse_part(gh)?;

        let val = u64::from_be_bytes([a, b, c, d, e, f, g, h])
            .try_into()
            .context("invalid id format")?;

        Ok(Self(val))
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let id = self
            .0
            .get()
            .to_be_bytes()
            .array_chunks()
            .map(|[a, b]| format!("{a:02x}{b:02x}"))
            .join("-");

        write!(f, "{}", id)
    }
}

impl fmt::Debug for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.to_string().fmt(f)
    }
}

impl<'de> Deserialize<'de> for Id {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let id = String::deserialize(deserializer)?;
        let id = Self::from_str(&id).map_err(D::Error::custom)?;

        Ok(id)
    }
}

impl Serialize for Id {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test() {
        let mut rng = ChaCha8Rng::from_seed(Default::default());
        let id = rng.gen::<Id>();

        assert_eq!("d640-5f89-2fef-003e", id.to_string());

        assert_eq!(
            "d640-5f89-2fef-003e",
            Id::from_str("d640-5f89-2fef-003e").unwrap().to_string()
        );

        assert_eq!(
            serde_json::Value::String("d640-5f89-2fef-003e".into()),
            serde_json::to_value(id).unwrap(),
        );

        assert_eq!(
            id,
            serde_json::from_value::<Id>(serde_json::Value::String(
                "d640-5f89-2fef-003e".into()
            ))
            .unwrap(),
        );
    }
}

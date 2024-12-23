use anyhow::{Context, Result};
use std::error::Error;
use std::str::FromStr;

pub fn entries<'a>(
    spec: &'a str,
) -> Box<dyn Iterator<Item = Result<SpecEntry<'a>>> + 'a> {
    if spec.is_empty() {
        return Box::new(Vec::new().into_iter());
    }

    Box::new(spec.split(",").map(|entry| {
        let (key, value) = entry
            .split_once("=")
            .with_context(|| format!("missing `=` near `{entry}`"))?;

        Ok(SpecEntry { key, value })
    }))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpecEntry<'a> {
    pub key: &'a str,
    pub value: &'a str,
}

impl SpecEntry<'_> {
    pub fn value<T>(self) -> Result<T>
    where
        T: FromStr,
        T::Err: Error + Send + Sync + 'static,
    {
        self.value
            .parse()
            .with_context(|| format!("couldn't parse `{}`", self.key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entries() {
        assert!(super::entries("").next().is_none());

        assert_eq!(
            vec![
                SpecEntry {
                    key: "one",
                    value: "1"
                },
                SpecEntry {
                    key: "two",
                    value: "2",
                },
                SpecEntry {
                    key: "three",
                    value: "3",
                }
            ],
            super::entries("one=1,two=2,three=3")
                .map(|entry| entry.unwrap())
                .collect::<Vec<_>>()
        );
    }
}

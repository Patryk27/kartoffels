mod v2;
mod v3;

use anyhow::{Context, Result};
use ciborium::Value;
use tracing::info;

pub fn run(old: u32, new: u32, mut world: Value) -> Result<Value> {
    let migrations = [v2::run, v3::run];

    for nth in old..new {
        info!("migrating: v{} -> v{}", nth, nth + 1);

        world = migrations[(nth - 1) as usize](world)
            .with_context(|| format!("migration v{} failed", nth + 1))?;
    }

    Ok(world)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{cbor_to_json, json_to_cbor};
    use pretty_assertions as pa;
    use std::fs;
    use std::path::Path;
    use test_case::test_case;

    #[test_case(2)]
    #[test_case(3)]
    fn test(version: u32) {
        let dir = Path::new("src")
            .join("store")
            .join("migrations")
            .join(format!("v{}", version))
            .join("test");

        let given_path = dir.join("given.json");
        let expected_path = dir.join("expected.json");
        let expected_new_path = dir.join("expected.json.new");

        let given = fs::read_to_string(&given_path).unwrap();
        let given = serde_json::from_str(&given).unwrap();
        let given = json_to_cbor(given);

        let actual = run(version - 1, version, given).unwrap();
        let actual = cbor_to_json(actual);
        let actual = serde_json::to_string_pretty(&actual).unwrap();

        let expected = fs::read_to_string(&expected_path)
            .unwrap_or_default()
            .trim()
            .to_owned();

        if expected == actual {
            _ = fs::remove_file(&expected_new_path);
        } else {
            _ = fs::write(&expected_new_path, &actual);
        }

        pa::assert_eq!(expected, actual);
    }
}

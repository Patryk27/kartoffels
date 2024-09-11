mod v2;
mod v3;
mod v4;
mod v5;

use crate::storage::VERSION;
use anyhow::Result;
use ciborium::Value;
use tracing::info;

const MIGRATIONS: [fn(&mut Value); (VERSION - 1) as usize] =
    [v2::run, v3::run, v4::run, v5::run];

pub fn run(old: u32, new: u32, mut world: Value) -> Result<Value> {
    for nth in old..new {
        info!("migrating: v{} -> v{}", nth, nth + 1);

        MIGRATIONS[(nth - 1) as usize](&mut world);
    }

    Ok(world)
}

#[cfg(test)]
mod tests {
    use kartoffels_utils::{cbor_to_json, json_to_cbor, Asserter};
    use std::fs;
    use std::path::Path;

    pub fn run(nth: u32) {
        let dir = Path::new("src")
            .join("storage")
            .join("migrations")
            .join(format!("v{}", nth))
            .join("test");

        let given_path = dir.join("given.json");

        let given = fs::read_to_string(&given_path).unwrap();
        let given = serde_json::from_str(&given).unwrap();
        let given = json_to_cbor(given);

        let actual = super::run(nth - 1, nth, given).unwrap();
        let actual = cbor_to_json(actual);
        let actual = serde_json::to_string_pretty(&actual).unwrap();

        Asserter::new(dir).assert("expected.json", actual);
    }
}
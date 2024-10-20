mod v02;
mod v03;
mod v04;
mod v05;
mod v06;
mod v07;
mod v08;
mod v09;
mod v10;
mod v11;

use crate::storage::VERSION;
use anyhow::Result;
use ciborium::Value;
use tracing::info;

const MIGRATIONS: [fn(&mut Value); (VERSION - 1) as usize] = [
    v02::run,
    v03::run,
    v04::run,
    v05::run,
    v06::run,
    v07::run,
    v08::run,
    v09::run,
    v10::run,
    v11::run,
];

pub fn run(old: u32, new: u32, mut world: Value) -> Result<Value> {
    for nth in old..new {
        info!("migrating: v{} -> v{}", nth, nth + 1);

        MIGRATIONS[(nth - 1) as usize](&mut world);
    }

    Ok(world)
}

#[cfg(test)]
mod tests {
    use kartoffels_utils::{cbor_to_json, json_to_cbor};
    use pretty_assertions as pa;

    pub fn run(nth: u32, given: &str, expected: &str) {
        let given = serde_json::from_str(given).unwrap();
        let given = json_to_cbor(given);

        let actual = super::run(nth - 1, nth, given).unwrap();
        let actual = cbor_to_json(actual, false);
        let actual = serde_json::to_string_pretty(&actual).unwrap();

        pa::assert_eq!(expected.trim(), actual.trim());
    }
}

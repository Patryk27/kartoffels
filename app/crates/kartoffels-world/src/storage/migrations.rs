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
mod v12;
mod v13;
mod v14;

use anyhow::Result;
use ciborium::Value;
use tracing::info;

static MIGRATIONS: &[fn(&mut Value)] = &[
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
    v12::run,
    v13::run,
    v14::run,
];

pub fn run(old: u32, new: u32, mut world: Value) -> Result<Value> {
    for idx in old..new {
        info!("migrating: v{} -> v{}", idx, idx + 1);

        MIGRATIONS[(idx - 1) as usize](&mut world);
    }

    Ok(world)
}

pub const fn version() -> u32 {
    MIGRATIONS.len() as u32 + 1
}

#[cfg(test)]
mod tests {
    use kartoffels_utils::{cbor_to_json, json_to_cbor};
    use pretty_assertions as pa;

    pub fn run(idx: u32, given: &str, expected: &str) {
        let given = serde_json::from_str(given).unwrap();
        let given = json_to_cbor(given);

        let expected = serde_json::from_str(expected).unwrap();
        let expected = json_to_cbor(expected);

        let actual = super::run(idx - 1, idx, given).unwrap();

        if expected != actual {
            let actual = cbor_to_json(actual, false);
            let actual = serde_json::to_string_pretty(&actual).unwrap();

            let expected = serde_json::to_string_pretty(&expected).unwrap();

            pa::assert_eq!(expected.trim(), actual.trim());
        }
    }
}

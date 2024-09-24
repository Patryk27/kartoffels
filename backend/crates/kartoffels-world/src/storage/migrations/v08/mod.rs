use ciborium::Value;
use kartoffels_utils::CborMapExt;

pub fn run(world: &mut Value) {
    world.as_map_mut().unwrap().add_entry(
        "clock",
        Value::Map(vec![
            (Value::Text("type".into()), Value::Text("auto".into())),
            (Value::Text("hz".into()), Value::Integer(64_000.into())),
            (Value::Text("steps".into()), Value::Integer(1_000.into())),
        ]),
    );
}

#[cfg(test)]
mod tests {
    use crate::storage::migrations;

    #[test]
    fn test() {
        migrations::tests::run(8);
    }
}

use ciborium::Value;
use kartoffels_utils::CborValueExt;

pub fn run(world: &mut Value) {
    world
        .query_mut("/policy")
        .next()
        .unwrap()
        .as_map_mut()
        .unwrap()
        .push((Value::Text("auto_respawn".into()), Value::Bool(true)));
}

#[cfg(test)]
mod tests {
    use crate::storage::migrations;

    #[test]
    fn test() {
        migrations::tests::run(5);
    }
}

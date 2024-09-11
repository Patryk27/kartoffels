use ciborium::Value;
use kartoffels_utils::CborValueExt;

pub fn run(world: &mut Value) {
    for obj in world.query_mut("/bots/alive/*") {
        obj.as_map_mut()
            .unwrap()
            .push((Value::Text("ephemeral".into()), Value::Bool(false)));
    }

    for obj in world.query_mut("/bots/queued/*") {
        let obj = obj.as_map_mut().unwrap();

        obj.push((Value::Text("pos".into()), Value::Null));
        obj.push((Value::Text("ephemeral".into()), Value::Bool(false)));
    }
}

#[cfg(test)]
mod tests {
    use crate::storage::migrations;

    #[test]
    fn test() {
        migrations::tests::run(4);
    }
}

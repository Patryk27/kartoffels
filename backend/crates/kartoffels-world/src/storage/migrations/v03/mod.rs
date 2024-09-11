use ciborium::Value;
use kartoffels_utils::CborValueExt;

pub fn run(world: &mut Value) {
    world
        .query_mut("/bots/dead")
        .next()
        .unwrap()
        .as_array_mut()
        .unwrap()
        .clear();

    for obj in world.query_mut("/bots/{alive,queued}/*") {
        obj.as_map_mut()
            .unwrap()
            .push((Value::Text("events".into()), Value::Array(vec![])));
    }
}

#[cfg(test)]
mod tests {
    use crate::storage::migrations;

    #[test]
    fn test() {
        migrations::tests::run(3);
    }
}

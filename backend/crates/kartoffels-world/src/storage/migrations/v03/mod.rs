use ciborium::Value;
use kartoffels_utils::{CborMapExt, CborValueExt};

pub fn run(world: &mut Value) {
    world
        .query_mut("/bots/dead")
        .next()
        .unwrap()
        .as_array_mut()
        .unwrap()
        .clear();

    for bot in world.query_mut("/bots/{alive,queued}/*") {
        bot.as_map_mut()
            .unwrap()
            .add_entry("events", Value::Array(vec![]));
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

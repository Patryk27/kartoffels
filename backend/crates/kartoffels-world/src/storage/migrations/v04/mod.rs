use ciborium::Value;
use kartoffels_utils::{CborMapExt, CborValueExt};

pub fn run(world: &mut Value) {
    for bot in world.query_mut("/bots/alive/*") {
        bot.as_map_mut()
            .unwrap()
            .add_entry("ephemeral", Value::Bool(false));
    }

    for bot in world.query_mut("/bots/queued/*") {
        bot.as_map_mut()
            .unwrap()
            .add_entry("pos", Value::Null)
            .add_entry("ephemeral", Value::Bool(false));
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

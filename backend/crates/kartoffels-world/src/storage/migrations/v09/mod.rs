use ciborium::Value;
use kartoffels_utils::{CborMapExt, CborValueExt};

pub fn run(world: &mut Value) {
    for bot in world.query_mut("/bots/alive/*") {
        let bot = bot.as_map_mut().unwrap();

        let dir = bot
            .get_entry_mut("motor")
            .unwrap()
            .as_map_mut()
            .unwrap()
            .remove_entry("dir")
            .unwrap();

        bot.add_entry("dir", dir);
    }
}

#[cfg(test)]
mod tests {
    use crate::storage::migrations;

    #[test]
    fn test() {
        migrations::tests::run(9);
    }
}

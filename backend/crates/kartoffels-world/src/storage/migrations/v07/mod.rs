use ciborium::Value;
use kartoffels_utils::{CborMapExt, CborValueExt};

pub fn run(world: &mut Value) {
    let theme = world
        .query_mut("/theme")
        .next()
        .unwrap()
        .as_map_mut()
        .unwrap();

    let config = theme.remove_entry("config").unwrap().into_map().unwrap();

    theme.extend(config);
}

#[cfg(test)]
mod tests {
    use crate::storage::migrations;

    #[test]
    fn test() {
        migrations::tests::run(7);
    }
}

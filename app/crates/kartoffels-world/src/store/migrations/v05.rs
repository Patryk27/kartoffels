use ciborium::Value;
use kartoffels_utils::{CborMapExt, CborValueExt};

pub fn run(world: &mut Value) {
    world
        .query_mut("/policy")
        .next()
        .unwrap()
        .as_map_mut()
        .unwrap()
        .add_entry("auto_respawn", Value::Bool(true));
}

#[cfg(test)]
mod tests {
    use crate::store::migrations;
    use indoc::indoc;

    #[test]
    fn test() {
        let given = indoc! {r#"
          {
            "policy": {
              "max_alive_bots": 16,
              "max_queued_bots": 32
            }
          }
        "#};

        let expected = indoc! {r#"
          {
            "policy": {
              "auto_respawn": true,
              "max_alive_bots": 16,
              "max_queued_bots": 32
            }
          }
        "#};

        migrations::tests::run(5, given, expected);
    }
}

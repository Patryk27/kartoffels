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
    use indoc::indoc;

    #[test]
    fn test() {
        let given = indoc! {r#"
          {
            "bots": {
              "alive": [
                {
                  "id": "0000-0000-0000-0001",
                  "motor": {
                    "cooldown": 123,
                    "dir": "^",
                    "vel": 0
                  },
                  "pos": [
                    12,
                    34
                  ]
                },
                {
                  "id": "0000-0000-0000-0002",
                  "motor": {
                    "cooldown": 321,
                    "dir": "^",
                    "vel": 0
                  },
                  "pos": [
                    56,
                    78
                  ]
                }
              ]
            }
          }
        "#};

        let expected = indoc! {r#"
          {
            "bots": {
              "alive": [
                {
                  "dir": "^",
                  "id": "0000-0000-0000-0001",
                  "motor": {
                    "cooldown": 123,
                    "vel": 0
                  },
                  "pos": [
                    12,
                    34
                  ]
                },
                {
                  "dir": "^",
                  "id": "0000-0000-0000-0002",
                  "motor": {
                    "cooldown": 321,
                    "vel": 0
                  },
                  "pos": [
                    56,
                    78
                  ]
                }
              ]
            }
          }
        "#};

        migrations::tests::run(9, given, expected);
    }
}

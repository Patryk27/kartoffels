use ciborium::Value;
use kartoffels_utils::{CborMapExt, CborValueExt};

pub fn run(world: &mut Value) {
    for bot in world.query_mut("/bots/{alive,queued}/*") {
        let bot = bot.as_map_mut().unwrap();

        let fw = bot
            .get_entry_mut("cpu")
            .unwrap()
            .as_map_mut()
            .unwrap()
            .remove_entry("fw")
            .unwrap();

        bot.add_entry("fw", fw);
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
                  "cpu": {
                    "fw": "foo",
                    "regs": 1
                  },
                  "id": 123
                },
                {
                  "cpu": {
                    "fw": "bar",
                    "regs": 2
                  },
                  "id": 321
                }
              ],
              "queued": [
                {
                  "cpu": {
                    "fw": "zar",
                    "regs": 3
                  },
                  "id": 234
                },
                {
                  "cpu": {
                    "fw": "dar",
                    "regs": 4
                  },
                  "id": 432
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
                  "cpu": {
                    "regs": 1
                  },
                  "fw": "foo",
                  "id": 123
                },
                {
                  "cpu": {
                    "regs": 2
                  },
                  "fw": "bar",
                  "id": 321
                }
              ],
              "queued": [
                {
                  "cpu": {
                    "regs": 3
                  },
                  "fw": "zar",
                  "id": 234
                },
                {
                  "cpu": {
                    "regs": 4
                  },
                  "fw": "dar",
                  "id": 432
                }
              ]
            }
          }
        "#};

        migrations::tests::run(11, given, expected);
    }
}

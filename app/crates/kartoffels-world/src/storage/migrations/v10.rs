use ciborium::Value;
use kartoffels_utils::{CborMapExt, CborValueExt};

pub fn run(world: &mut Value) {
    for bot in world.query_mut("/bots/{alive,queued}/*") {
        bot.as_map_mut()
            .unwrap()
            .add_entry("oneshot", Value::Bool(false));
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
                  "id": 123
                },
                {
                  "id": 321
                }
              ],
              "queued": [
                {
                  "id": 234
                },
                {
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
                  "id": 123,
                  "oneshot": false
                },
                {
                  "id": 321,
                  "oneshot": false
                }
              ],
              "queued": [
                {
                  "id": 234,
                  "oneshot": false
                },
                {
                  "id": 432,
                  "oneshot": false
                }
              ]
            }
          }
        "#};

        migrations::tests::run(10, given, expected);
    }
}

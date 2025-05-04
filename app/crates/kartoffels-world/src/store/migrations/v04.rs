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
    use crate::store::migrations;
    use indoc::indoc;

    #[test]
    fn test() {
        let given = indoc! {r#"
          {
            "bots": {
              "alive": [
                {
                  "id": "0000-0000-0000-0003"
                },
                {
                  "id": "0000-0000-0000-0004"
                }
              ],
              "queued": [
                {
                  "id": "0000-0000-0000-0003"
                },
                {
                  "id": "0000-0000-0000-0004"
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
                  "ephemeral": false,
                  "id": "0000-0000-0000-0003"
                },
                {
                  "ephemeral": false,
                  "id": "0000-0000-0000-0004"
                }
              ],
              "queued": [
                {
                  "ephemeral": false,
                  "id": "0000-0000-0000-0003",
                  "pos": null
                },
                {
                  "ephemeral": false,
                  "id": "0000-0000-0000-0004",
                  "pos": null
                }
              ]
            }
          }
        "#};

        migrations::tests::run(4, given, expected);
    }
}

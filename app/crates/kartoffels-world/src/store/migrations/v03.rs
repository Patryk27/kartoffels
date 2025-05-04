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
    use crate::store::migrations;
    use indoc::indoc;

    #[test]
    fn test() {
        let given = indoc! {r#"
          {
            "bots": {
              "alive": [
                {
                  "id": "0000-0000-0000-0001"
                },
                {
                  "id": "0000-0000-0000-0002"
                }
              ],
              "queued": [
                {
                  "id": "0000-0000-0000-0003"
                },
                {
                  "id": "0000-0000-0000-0004"
                }
              ],
              "dead": [
                {
                  "id": "0000-0000-0000-0005"
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
                  "events": [],
                  "id": "0000-0000-0000-0001"
                },
                {
                  "events": [],
                  "id": "0000-0000-0000-0002"
                }
              ],
              "dead": [],
              "queued": [
                {
                  "events": [],
                  "id": "0000-0000-0000-0003"
                },
                {
                  "events": [],
                  "id": "0000-0000-0000-0004"
                }
              ]
            }
          }
        "#};

        migrations::tests::run(3, given, expected);
    }
}

use ciborium::Value;
use kartoffels_utils::{CborMapExt, CborValueExt};

pub fn run(world: &mut Value) {
    for serial in world.query_mut("/bots/{alive,queued}/*/serial") {
        serial
            .as_map_mut()
            .unwrap()
            .rename_entry("buffer", "curr")
            .add_entry("next", Value::Array(vec![]))
            .add_entry("buffering", Value::Bool(false));
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
                  "serial": {
                    "buffer": [
                      1,
                      2,
                      3
                    ]
                  }
                }
              ],
              "queued": [
                {
                  "id": "0000-0000-0000-0002",
                  "serial": {
                    "buffer": [
                      4,
                      5,
                      6
                    ]
                  }
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
                  "id": "0000-0000-0000-0001",
                  "serial": {
                    "curr": [
                      1,
                      2,
                      3
                    ],
                    "next": [ ],
                    "buffering": false
                  }
                }
              ],
              "queued": [
                {
                  "id": "0000-0000-0000-0002",
                  "serial": {
                    "curr": [
                      4,
                      5,
                      6
                    ],
                    "next": [ ],
                    "buffering": false
                  }
                }
              ]
            }
          }
        "#};

        migrations::tests::run(16, given, expected);
    }
}

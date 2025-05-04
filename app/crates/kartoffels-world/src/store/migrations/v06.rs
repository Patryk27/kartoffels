use ciborium::Value;
use kartoffels_utils::{CborMapExt, CborValueExt};

pub fn run(world: &mut Value) {
    for bot in world.query_mut("/bots/{alive,queued}/*") {
        bot.as_map_mut().unwrap().rename_entry("vm", "cpu");
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
                  "id": "0000-0000-0000-0001",
                  "vm": {
                    "something": 123
                  }
                },
                {
                  "id": "0000-0000-0000-0002",
                  "vm": {
                    "something": 321
                  }
                }
              ],
              "queued": [
                {
                  "id": "0000-0000-0000-0003",
                  "vm": {
                    "something": true
                  }
                },
                {
                  "id": "0000-0000-0000-0004",
                  "vm": {
                    "something": false
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
                  "cpu": {
                    "something": 123
                  },
                  "id": "0000-0000-0000-0001"
                },
                {
                  "cpu": {
                    "something": 321
                  },
                  "id": "0000-0000-0000-0002"
                }
              ],
              "queued": [
                {
                  "cpu": {
                    "something": true
                  },
                  "id": "0000-0000-0000-0003"
                },
                {
                  "cpu": {
                    "something": false
                  },
                  "id": "0000-0000-0000-0004"
                }
              ]
            }
          }
        "#};

        migrations::tests::run(6, given, expected);
    }
}

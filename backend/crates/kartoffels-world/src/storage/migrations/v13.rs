use ciborium::Value;
use kartoffels_utils::{CborMapExt, CborValueExt};

pub fn run(world: &mut Value) {
    for bot in world.query_mut("/bots/alive/*") {
        bot.as_map_mut().unwrap().add_entry(
            "inventory",
            Value::Map(
                Vec::default()
                    .with_entry("objects", Value::Array(vec![Value::Null; 32])),
            ),
        );
    }

        bot.as_map_mut()
            .unwrap()
            .add_entry("inventory", Value::Array(vec![Value::Null; 32]));
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
                  "id": "1234-1234-1234-1234"
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
                  "id": "1234-1234-1234-1234",
                  "inventory": {
                    "objects": [
                      null, null, null, null, null, null, null, null,
                      null, null, null, null, null, null, null, null,
                      null, null, null, null, null, null, null, null,
                      null, null, null, null, null, null, null, null
                    ]
                  }
                }
                }
              ]
            }
          }
        "#};

        migrations::tests::run(13, given, expected);
    }
}

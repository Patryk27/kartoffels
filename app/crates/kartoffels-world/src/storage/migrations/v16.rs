use ciborium::Value;
use kartoffels_utils::{CborMapExt, CborValueExt};

pub fn run(world: &mut Value) {
    for radar in world.query_mut("/bots/alive/*/radar") {
        radar.as_map_mut().unwrap().rename_entry("scan", "memory");
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
                  "id": "1234-1234-1234-1234",
                  "radar": {
                    "scan": "something"
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
                  "id": "1234-1234-1234-1234",
                  "radar": {
                    "memory": "something"
                  }
                }
              ]
            }
          }
        "#};

        migrations::tests::run(16, given, expected);
    }
}

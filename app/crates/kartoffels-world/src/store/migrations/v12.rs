use ciborium::value::Integer;
use ciborium::Value;
use kartoffels_utils::{CborMapExt, CborValueExt};

pub fn run(world: &mut Value) {
    for bot in world.query_mut("/bots/alive/*") {
        let radar = bot
            .as_map_mut()
            .unwrap()
            .get_entry_mut("radar")
            .unwrap()
            .as_map_mut()
            .unwrap();

        let mut scan =
            radar.remove_entry("payload").unwrap().into_array().unwrap();

        assert_eq!(81, scan.len());

        for _ in 0..(81 * 2) {
            scan.push(Value::Integer(Integer::from(0)));
        }

        radar.add_entry("scan", Value::Array(scan));
        radar.remove_entry("pending_scan").unwrap();
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
                  "radar": {
                    "payload": [
                      1, 2, 3, 4, 5, 6, 7, 8, 9,
                      1, 2, 3, 4, 5, 6, 7, 8, 9,
                      1, 2, 3, 4, 5, 6, 7, 8, 9,
                      1, 2, 3, 4, 5, 6, 7, 8, 9,
                      1, 2, 3, 4, 5, 6, 7, 8, 9,
                      1, 2, 3, 4, 5, 6, 7, 8, 9,
                      1, 2, 3, 4, 5, 6, 7, 8, 9,
                      1, 2, 3, 4, 5, 6, 7, 8, 9,
                      1, 2, 3, 4, 5, 6, 7, 8, 9
                    ],
                    "cooldown": 123,
                    "pending_scan": 7
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
                  "radar": {
                    "scan": [
                      1, 2, 3, 4, 5, 6, 7, 8, 9,
                      1, 2, 3, 4, 5, 6, 7, 8, 9,
                      1, 2, 3, 4, 5, 6, 7, 8, 9,
                      1, 2, 3, 4, 5, 6, 7, 8, 9,
                      1, 2, 3, 4, 5, 6, 7, 8, 9,
                      1, 2, 3, 4, 5, 6, 7, 8, 9,
                      1, 2, 3, 4, 5, 6, 7, 8, 9,
                      1, 2, 3, 4, 5, 6, 7, 8, 9,
                      1, 2, 3, 4, 5, 6, 7, 8, 9,

                      0, 0, 0, 0, 0, 0, 0, 0, 0,
                      0, 0, 0, 0, 0, 0, 0, 0, 0,
                      0, 0, 0, 0, 0, 0, 0, 0, 0,
                      0, 0, 0, 0, 0, 0, 0, 0, 0,
                      0, 0, 0, 0, 0, 0, 0, 0, 0,
                      0, 0, 0, 0, 0, 0, 0, 0, 0,
                      0, 0, 0, 0, 0, 0, 0, 0, 0,
                      0, 0, 0, 0, 0, 0, 0, 0, 0,
                      0, 0, 0, 0, 0, 0, 0, 0, 0,

                      0, 0, 0, 0, 0, 0, 0, 0, 0,
                      0, 0, 0, 0, 0, 0, 0, 0, 0,
                      0, 0, 0, 0, 0, 0, 0, 0, 0,
                      0, 0, 0, 0, 0, 0, 0, 0, 0,
                      0, 0, 0, 0, 0, 0, 0, 0, 0,
                      0, 0, 0, 0, 0, 0, 0, 0, 0,
                      0, 0, 0, 0, 0, 0, 0, 0, 0,
                      0, 0, 0, 0, 0, 0, 0, 0, 0,
                      0, 0, 0, 0, 0, 0, 0, 0, 0
                    ],
                    "cooldown": 123
                  }
                }
              ]
            }
          }
        "#};

        migrations::tests::run(12, given, expected);
    }
}

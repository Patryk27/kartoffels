use ciborium::Value;
use kartoffels_utils::{CborMapExt, CborValueExt};

pub fn run(world: &mut Value) {
    let theme = world
        .query_mut("/theme")
        .next()
        .unwrap()
        .as_map_mut()
        .unwrap();

    let config = theme.remove_entry("config").unwrap().into_map().unwrap();

    theme.extend(config);
}

#[cfg(test)]
mod tests {
    use crate::store::migrations;
    use indoc::indoc;

    #[test]
    fn test() {
        let given = indoc! {r#"
          {
            "theme": {
              "config": {
                "size": [
                  80,
                  80
                ]
              },
              "type": "dungeon"
            }
          }
        "#};

        let expected = indoc! {r#"
          {
            "theme": {
              "size": [
                80,
                80
              ],
              "type": "dungeon"
            }
          }
        "#};

        migrations::tests::run(7, given, expected);
    }
}

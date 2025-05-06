use ciborium::Value;
use kartoffels_utils::CborMapExt;

pub fn run(world: &mut Value) {
    world.as_map_mut().unwrap().add_entry(
        "clock",
        Vec::new()
            .with_entry("type", "auto")
            .with_entry("hz", 64_000)
            .with_entry("steps", 1_000)
            .into_map(),
    );
}

#[cfg(test)]
mod tests {
    use crate::store::migrations;
    use indoc::indoc;

    #[test]
    fn test() {
        let given = indoc! {r#"
          {
            "bots": "something something foo",
            "theme": "something something bar"
          }
        "#};

        let expected = indoc! {r#"
          {
            "bots": "something something foo",
            "clock": {
              "hz": 64000,
              "steps": 1000,
              "type": "auto"
            },
            "theme": "something something bar"
          }
        "#};

        migrations::tests::run(8, given, expected);
    }
}

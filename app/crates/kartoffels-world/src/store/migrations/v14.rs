use ciborium::Value;
use kartoffels_utils::CborMapExt;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

pub fn run(world: &mut Value) {
    #[cfg(test)]
    let rng = ChaCha8Rng::from_seed(Default::default());

    #[cfg(not(test))]
    let rng = ChaCha8Rng::from_entropy();

    world.as_map_mut().unwrap().add_entry("rng", rng);
}

#[cfg(test)]
mod tests {
    use crate::store::migrations;
    use indoc::indoc;

    #[test]
    fn test() {
        let given = indoc! {r#"
          { }
        "#};

        let expected = indoc! {r#"
          {
            "rng": {
              "seed": [
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0
              ],
              "stream": 0,
              "word_pos": 0
            }
          }
        "#};

        migrations::tests::run(14, given, expected);
    }
}

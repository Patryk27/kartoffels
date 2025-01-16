use ciborium::Value;
use kartoffels_utils::{CborMapExt, CborValueExt};

pub fn run(world: &mut Value) {
    let now = {
        #[cfg(test)]
        let now = chrono::DateTime::from_timestamp(0, 0);

        // Since we didn't store anything that would allow us to recover
        // `born_at` timestamps, pretend all bots got born now.
        //
        // No biggie, all bots get killed eventually and then they'll get the
        // correct, updated timestamps.
        #[cfg(not(test))]
        let now = chrono::Utc::now();

        Value::serialized(&now).unwrap()
    };

    let scores = world
        .as_map_mut()
        .unwrap()
        .remove_entry("mode")
        .unwrap()
        .as_map_mut()
        .unwrap()
        .remove_entry("scores")
        .unwrap()
        .into_map()
        .unwrap();

    let mut lives: Vec<_> = scores
        .into_iter()
        .map(|(id, score)| {
            let life = Vec::default()
                .with_entry(
                    "curr",
                    Value::Map(
                        Vec::default()
                            .with_entry("score", score)
                            .with_entry("born_at", now.clone()),
                    ),
                )
                .with_entry("prev", Value::Array(Vec::default()))
                .with_entry("len", Value::Integer(1.into()));

            (id, Value::Map(life))
        })
        .collect();

    // `mode.scores` only kept positive data, i.e. bots with zero kills were not
    // present there - but our `lives` table must contain entries for all alive
    // bots as well; so to avoid crashes, lets's go through `/bots/alive/*/id`
    // and add zeroed-out scores for bots that weren't present in `mode.scores`.
    for alive_bot_id in world.query_mut("/bots/alive/*/id") {
        if lives.iter().any(|(bot_id, _)| {
            bot_id.as_text().unwrap() == alive_bot_id.as_text().unwrap()
        }) {
            continue;
        }

        let life = Vec::default()
            .with_entry(
                "curr",
                Value::Map(
                    Vec::default()
                        .with_entry("score", Value::Integer(0.into()))
                        .with_entry("born_at", now.clone()),
                ),
            )
            .with_entry("prev", Value::Array(Vec::default()))
            .with_entry("len", Value::Integer(1.into()));

        lives.push((alive_bot_id.clone(), Value::Map(life)));
    }

    world
        .as_map_mut()
        .unwrap()
        .add_entry("lives", Value::Map(lives));
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
                  "id": "0000-0000-0000-0001"
                },
                {
                  "id": "0000-0000-0000-0002"
                },
                {
                  "id": "0000-0000-0000-0003"
                },
                {
                  "id": "0000-0000-0000-0004"
                }
              ]
            },
            "mode": {
              "scores": {
                "0000-0000-0000-0001": 100,
                "0000-0000-0000-0002": 200,
                "0000-0000-0000-0004": 300
              }
            }
          }
        "#};

        let expected = indoc! {r#"
          {
            "bots": {
              "alive": [
                {
                  "id": "0000-0000-0000-0001"
                },
                {
                  "id": "0000-0000-0000-0002"
                },
                {
                  "id": "0000-0000-0000-0003"
                },
                {
                  "id": "0000-0000-0000-0004"
                }
              ]
            },
            "lives": {
              "0000-0000-0000-0001": {
                "curr": {
                  "score": 100,
                  "born_at": "1970-01-01T00:00:00Z"
                },
                "prev": [],
                "len": 1
              },
              "0000-0000-0000-0002": {
                "curr": {
                  "score": 200,
                  "born_at": "1970-01-01T00:00:00Z"
                },
                "prev": [],
                "len": 1
              },
              "0000-0000-0000-0004": {
                "curr": {
                  "score": 300,
                  "born_at": "1970-01-01T00:00:00Z"
                },
                "prev": [],
                "len": 1
              },
              "0000-0000-0000-0003": {
                "curr": {
                  "score": 0,
                  "born_at": "1970-01-01T00:00:00Z"
                },
                "prev": [],
                "len": 1
              }
            }
          }
        "#};

        migrations::tests::run(15, given, expected);
    }
}

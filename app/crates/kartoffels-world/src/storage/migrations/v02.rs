use ciborium::value::Integer;
use ciborium::Value;
use kartoffels_utils::CborValueExt;

pub fn run(world: &mut Value) {
    let i0 = Value::Integer(Integer::from(0));
    let i1p = Value::Integer(Integer::from(1));
    let i1n = Value::Integer(Integer::from(-1));

    let up = Value::Array(vec![i0.clone(), i1n.clone()]);
    let down = Value::Array(vec![i0.clone(), i1p.clone()]);
    let left = Value::Array(vec![i1n.clone(), i0.clone()]);
    let right = Value::Array(vec![i1p.clone(), i0.clone()]);

    for obj in world.query_mut("/bots/{alive,queued}/*/motor/dir") {
        if *obj == up {
            *obj = Value::Text("^".into());
        } else if *obj == down {
            *obj = Value::Text("v".into());
        } else if *obj == left {
            *obj = Value::Text("<".into());
        } else if *obj == right {
            *obj = Value::Text(">".into());
        } else {
            unreachable!();
        }
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
                  "motor": {
                    "cooldown": 0,
                    "dir": [
                      1,
                      0
                    ],
                    "vel": 0
                  }
                },
                {
                  "id": "0000-0000-0000-0002",
                  "motor": {
                    "cooldown": 0,
                    "dir": [
                      -1,
                      0
                    ],
                    "vel": 0
                  }
                }
              ],
              "queued": [
                {
                  "id": "0000-0000-0000-0003",
                  "motor": {
                    "cooldown": 0,
                    "dir": [
                      0,
                      1
                    ],
                    "vel": 0
                  }
                },
                {
                  "id": "0000-0000-0000-0004",
                  "motor": {
                    "cooldown": 0,
                    "dir": [
                      0,
                      -1
                    ],
                    "vel": 0
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
                  "motor": {
                    "cooldown": 0,
                    "dir": ">",
                    "vel": 0
                  }
                },
                {
                  "id": "0000-0000-0000-0002",
                  "motor": {
                    "cooldown": 0,
                    "dir": "<",
                    "vel": 0
                  }
                }
              ],
              "queued": [
                {
                  "id": "0000-0000-0000-0003",
                  "motor": {
                    "cooldown": 0,
                    "dir": "v",
                    "vel": 0
                  }
                },
                {
                  "id": "0000-0000-0000-0004",
                  "motor": {
                    "cooldown": 0,
                    "dir": "^",
                    "vel": 0
                  }
                }
              ]
            }
          }
        "#};

        migrations::tests::run(2, given, expected);
    }
}

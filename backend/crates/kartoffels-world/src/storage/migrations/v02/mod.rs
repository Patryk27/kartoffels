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

    #[test]
    fn test() {
        migrations::tests::run(2);
    }
}

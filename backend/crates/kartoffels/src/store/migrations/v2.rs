use crate::CborValueExt;
use anyhow::{anyhow, Result};
use ciborium::value::Integer;
use ciborium::Value;

pub fn run(mut world: Value) -> Result<Value> {
    let i0 = Value::Integer(Integer::from(0));
    let i1p = Value::Integer(Integer::from(1));
    let i1n = Value::Integer(Integer::from(-1));

    let up = Value::Array(vec![i0.clone(), i1n.clone()]);
    let down = Value::Array(vec![i0.clone(), i1p.clone()]);
    let left = Value::Array(vec![i1n.clone(), i0.clone()]);
    let right = Value::Array(vec![i1p.clone(), i0.clone()]);

    for entry in world.query_mut("/bots/{alive,queued}/*/motor/dir") {
        if *entry == up {
            *entry = Value::Text("^".into());
        } else if *entry == down {
            *entry = Value::Text("v".into());
        } else if *entry == left {
            *entry = Value::Text("<".into());
        } else if *entry == right {
            *entry = Value::Text(">".into());
        } else {
            return Err(anyhow!("invalid direction: {:?}", entry));
        }
    }

    Ok(world)
}

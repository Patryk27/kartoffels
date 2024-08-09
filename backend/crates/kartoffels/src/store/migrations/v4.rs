use crate::CborValueExt;
use anyhow::{Context, Result};
use ciborium::Value;

pub fn run(mut world: Value) -> Result<Value> {
    for obj in world.query_mut("/bots/queued/*") {
        obj.as_map_mut()
            .context("expected an object")?
            .push((Value::Text("pos".into()), Value::Null));
    }

    Ok(world)
}

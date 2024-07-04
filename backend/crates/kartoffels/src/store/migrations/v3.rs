use crate::CborValueExt;
use anyhow::{Context, Result};
use ciborium::Value;

pub fn run(mut world: Value) -> Result<Value> {
    world
        .query_mut("/bots/dead")
        .next()
        .context("missing object: /bots/dead")?
        .as_array_mut()
        .context("expected an array")?
        .clear();

    for obj in world.query_mut("/bots/{alive,queued}/*") {
        obj.as_map_mut()
            .context("expected an object")?
            .push((Value::Text("events".into()), Value::Array(vec![])));
    }

    Ok(world)
}

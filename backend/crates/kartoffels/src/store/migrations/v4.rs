use crate::CborValueExt;
use anyhow::{Context, Result};
use ciborium::Value;

pub fn run(mut world: Value) -> Result<Value> {
    for obj in world.query_mut("/bots/alive/*") {
        obj.as_map_mut()
            .context("expected an object")?
            .push((Value::Text("ephemeral".into()), Value::Bool(false)));
    }

    for obj in world.query_mut("/bots/queued/*") {
        let obj = obj.as_map_mut().context("expected an object")?;

        obj.push((Value::Text("pos".into()), Value::Null));
        obj.push((Value::Text("ephemeral".into()), Value::Bool(false)));
    }

    Ok(world)
}

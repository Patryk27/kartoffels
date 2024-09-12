use ciborium::Value;
use kartoffels_utils::CborValueExt;

pub fn run(world: &mut Value) {
    for obj in world.query_mut("/bots/{alive,queued}/*") {
        let obj = obj.as_map_mut().unwrap();

        let (_, vm) = obj
            .extract_if(|(key, _)| key.as_text().unwrap() == "vm")
            .next()
            .unwrap();

        obj.push((Value::Text("cpu".into()), vm));
    }
}

#[cfg(test)]
mod tests {
    use crate::storage::migrations;

    #[test]
    fn test() {
        migrations::tests::run(6);
    }
}

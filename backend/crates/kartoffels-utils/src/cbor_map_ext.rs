use ciborium::Value;

pub trait CborMapExt
where
    Self: Sized,
{
    fn add_entry(&mut self, key: &str, val: Value) -> &mut Self;
    fn get_entry_mut(&mut self, key: &str) -> Option<&mut Value>;
    fn remove_entry(&mut self, key: &str) -> Option<Value>;

    fn with_entry(mut self, key: &str, val: Value) -> Self {
        self.add_entry(key, val);
        self
    }

    fn rename_entry(&mut self, from_key: &str, to_key: &str) -> &mut Self {
        if let Some(val) = self.remove_entry(from_key) {
            self.add_entry(to_key, val)
        } else {
            self
        }
    }
}

impl CborMapExt for Vec<(Value, Value)> {
    fn add_entry(&mut self, key: &str, val: Value) -> &mut Self {
        self.push((Value::Text(key.into()), val));
        self
    }

    fn get_entry_mut(&mut self, key: &str) -> Option<&mut Value> {
        self.iter_mut().find_map(|(entry_key, entry_val)| {
            if entry_key.as_text().unwrap() == key {
                Some(entry_val)
            } else {
                None
            }
        })
    }

    fn remove_entry(&mut self, key: &str) -> Option<Value> {
        self.extract_if(|(entry_key, _)| entry_key.as_text().unwrap() == key)
            .next()
            .map(|(_, val)| val)
    }
}

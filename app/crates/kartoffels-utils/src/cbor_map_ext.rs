use ciborium::Value;
use serde::Serialize;

pub trait CborMapExt
where
    Self: Sized,
{
    fn add_entry(&mut self, key: &str, value: impl Serialize) -> &mut Self;
    fn get_entry_mut(&mut self, key: &str) -> Option<&mut Value>;
    fn remove_entry(&mut self, key: &str) -> Option<Value>;

    fn with_entry(mut self, key: &str, value: impl Serialize) -> Self {
        self.add_entry(key, value);
        self
    }

    fn rename_entry(&mut self, old: &str, new: &str) -> &mut Self {
        if let Some(value) = self.remove_entry(old) {
            self.add_entry(new, value)
        } else {
            self
        }
    }

    fn into_map(self) -> Value;
}

impl CborMapExt for Vec<(Value, Value)> {
    fn add_entry(&mut self, key: &str, value: impl Serialize) -> &mut Self {
        let value = Value::serialized(&value).unwrap();

        self.push((Value::Text(key.into()), value));
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
        self.extract_if(.., |(entry_key, _)| {
            entry_key.as_text().unwrap() == key
        })
        .next()
        .map(|(_, val)| val)
    }

    fn into_map(self) -> Value {
        Value::Map(self)
    }
}

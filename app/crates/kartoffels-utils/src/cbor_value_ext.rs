use ciborium::Value;
use std::iter;

pub trait CborValueExt {
    fn query_mut<'a>(
        &'a mut self,
        query: &'a str,
    ) -> Box<dyn Iterator<Item = &'a mut Value> + 'a>;
}

impl CborValueExt for Value {
    fn query_mut<'a>(
        &'a mut self,
        query: &'a str,
    ) -> Box<dyn Iterator<Item = &'a mut Value> + 'a> {
        match query.split_once('/') {
            Some((key, query)) => match key {
                "" => self.query_mut(query),

                "*" => Box::new(
                    self.as_array_mut()
                        .into_iter()
                        .flatten()
                        .flat_map(|this| this.query_mut(query)),
                ),

                key => Box::new(
                    lookup_mut(self, key)
                        .flat_map(|this| this.query_mut(query)),
                ),
            },

            None => Box::new(lookup_mut(self, query)),
        }
    }
}

fn lookup_mut<'a>(
    value: &'a mut Value,
    query: &'a str,
) -> Box<dyn Iterator<Item = &'a mut Value> + 'a> {
    match value {
        Value::Array(value) => Box::new(lookup_mut_array(value, query)),
        Value::Map(value) => Box::new(lookup_mut_map(value, query)),
        _ => Box::new(iter::empty()),
    }
}

fn lookup_mut_array<'a>(
    value: &'a mut [Value],
    query: &'a str,
) -> impl Iterator<Item = &'a mut Value> {
    if query == "*" {
        value.iter_mut()
    } else {
        todo!();
    }
}

fn lookup_mut_map<'a>(
    value: &'a mut [(Value, Value)],
    query: &'a str,
) -> impl Iterator<Item = &'a mut Value> + 'a {
    let keys = if query == "*" {
        None
    } else if query.starts_with('{') {
        let key = query.strip_prefix('{').unwrap();
        let key = key.strip_suffix('}').unwrap();

        Some(key.split(',').collect())
    } else {
        Some(vec![query])
    };

    value.iter_mut().filter_map(move |(curr_key, curr_val)| {
        let curr_key = curr_key.as_text().unwrap_or_default();

        let curr_key_matches =
            keys.as_ref().is_none_or(|keys| keys.contains(&curr_key));

        if curr_key_matches {
            Some(curr_val)
        } else {
            None
        }
    })
}

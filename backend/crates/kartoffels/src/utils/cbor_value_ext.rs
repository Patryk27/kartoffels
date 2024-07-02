use ciborium::Value;

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

                key if key.starts_with('{') => {
                    let key = key.strip_prefix('{').unwrap();
                    let key = key.strip_suffix('}').unwrap();
                    let keys = key.split(',').collect();

                    Box::new(
                        lookup_mut(self, keys)
                            .flat_map(|this| this.query_mut(query)),
                    )
                }

                key => Box::new(
                    lookup_mut(self, vec![key])
                        .flat_map(|this| this.query_mut(query)),
                ),
            },

            None => Box::new(lookup_mut(self, vec![query])),
        }
    }
}

fn lookup_mut<'a>(
    val: &'a mut Value,
    keys: Vec<&'a str>,
) -> impl Iterator<Item = &'a mut Value> + 'a {
    val.as_map_mut().into_iter().flatten().filter_map(
        move |(curr_key, curr_val)| {
            let curr_key = curr_key.as_text().unwrap_or_default();

            if keys.contains(&curr_key) {
                Some(curr_val)
            } else {
                None
            }
        },
    )
}

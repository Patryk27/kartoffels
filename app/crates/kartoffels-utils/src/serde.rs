pub mod sorted_map {
    use itertools::Itertools;
    use serde::{Serialize, Serializer};
    use std::collections::{BTreeMap, HashMap};

    pub fn serialize<S, K, V, St>(
        values: &HashMap<K, V, St>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        K: Serialize + Ord,
        V: Serialize,
    {
        values
            .iter()
            .sorted_by_key(|v| v.0)
            .collect::<BTreeMap<_, _>>()
            .serialize(serializer)
    }
}

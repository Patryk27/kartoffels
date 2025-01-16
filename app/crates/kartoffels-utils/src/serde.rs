#![allow(dead_code)]

pub mod instant {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::{Duration, Instant};

    pub fn serialize<S>(
        instant: &Instant,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        (*instant - Instant::now())
            .as_secs_f32()
            .serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Instant, D::Error>
    where
        D: Deserializer<'de>,
    {
        let duration = f32::deserialize(deserializer)?;
        let duration = Duration::from_secs_f32(duration);

        Ok(Instant::now() + duration)
    }
}

pub mod instant_opt {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::{Duration, Instant};

    pub fn serialize<S>(
        instant: &Option<Instant>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(instant) = instant {
            super::instant::serialize(instant, serializer)
        } else {
            serializer.serialize_none()
        }
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<Option<Instant>, D::Error>
    where
        D: Deserializer<'de>,
    {
        if let Some(duration) = Option::deserialize(deserializer)? {
            Ok(Some(Instant::now() + Duration::from_secs_f32(duration)))
        } else {
            Ok(None)
        }
    }
}

use crate::{BotId, DeadBot};
use lru::LruCache;
use maybe_owned::MaybeOwned;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::num::NonZeroUsize;

#[derive(Clone, Debug)]
pub struct DeadBots {
    entries: LruCache<BotId, DeadBot>,
}

impl DeadBots {
    const MAX_ENTRIES: usize = 4 * 1024;

    pub fn add(&mut self, id: BotId, bot: DeadBot) {
        self.entries.put(id, bot);
    }

    pub fn remove(&mut self, id: BotId) {
        self.entries.pop_entry(&id);
    }

    pub fn contains(&self, id: BotId) -> bool {
        self.entries.contains(&id)
    }
}

impl Default for DeadBots {
    fn default() -> Self {
        Self {
            entries: LruCache::new(
                NonZeroUsize::new(Self::MAX_ENTRIES).unwrap(),
            ),
        }
    }
}

impl Serialize for DeadBots {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_seq(self.entries.iter().map(|(id, bot)| {
            SerializedDeadBot {
                id: *id,
                bot: MaybeOwned::Borrowed(bot),
            }
        }))
    }
}

impl<'de> Deserialize<'de> for DeadBots {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut this = Self::default();
        let bots = Vec::<SerializedDeadBot>::deserialize(deserializer)?;

        for bot in bots {
            this.add(bot.id, bot.bot.into_owned());
        }

        Ok(this)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct SerializedDeadBot<'a> {
    id: BotId,

    #[serde(flatten)]
    bot: MaybeOwned<'a, DeadBot>,
}

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
    const MAX_ENTRIES: usize = 32 * 1024;

    pub fn add(&mut self, id: BotId, bot: DeadBot) {
        self.entries.put(id, bot);
    }

    pub fn get(&self, id: BotId) -> Option<&DeadBot> {
        self.entries.peek(&id)
    }

    pub fn remove(&mut self, id: BotId) {
        self.entries.pop_entry(&id);
    }

    pub fn has(&self, id: BotId) -> bool {
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
        let proxy = SerializedDeadBots {
            bots: self
                .entries
                .iter()
                .map(|(id, bot)| SerializedDeadBot {
                    id: *id,
                    bot: MaybeOwned::Borrowed(bot),
                })
                .collect(),
        };

        proxy.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for DeadBots {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut this = Self::default();
        let proxy = SerializedDeadBots::deserialize(deserializer)?;

        for entry in proxy.bots {
            this.add(entry.id, entry.bot.into_owned());
        }

        Ok(this)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
struct SerializedDeadBots<'a> {
    bots: Vec<SerializedDeadBot<'a>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SerializedDeadBot<'a> {
    id: BotId,

    #[serde(flatten)]
    bot: MaybeOwned<'a, DeadBot>,
}

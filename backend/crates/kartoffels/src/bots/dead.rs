use crate::{BotId, DeadBot};
use maybe_owned::MaybeOwned;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct DeadBots {
    entries: HashMap<BotId, DeadBot>,
}

impl DeadBots {
    pub fn add(&mut self, id: BotId, bot: DeadBot) {
        self.entries.insert(id, bot);
    }

    pub fn remove(&mut self, id: BotId) {
        self.entries.remove(&id);
    }

    pub fn has(&self, id: BotId) -> bool {
        self.entries.contains_key(&id)
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

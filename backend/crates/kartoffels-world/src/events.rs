use crate::BotId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    BotCreated { id: BotId },
    BotKilled { id: BotId },
}

impl Event {
    pub fn is_bot_killed(&self, id: BotId) -> bool {
        *self == Event::BotKilled { id }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_bot_killed() {
        let event1 = Event::BotKilled {
            id: BotId::new(123),
        };

        let event2 = Event::BotCreated {
            id: BotId::new(123),
        };

        assert!(event1.is_bot_killed(BotId::new(123)));
        assert!(!event1.is_bot_killed(BotId::new(321)));
        assert!(!event2.is_bot_killed(BotId::new(321)));
    }
}

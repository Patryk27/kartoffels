use crate::{Session, SessionEntry, SessionId};
use ahash::AHashMap;
use std::collections::hash_map;
use std::sync::{Arc, Mutex};
use tracing::info;

#[derive(Debug, Default)]
pub struct Sessions {
    entries: Arc<Mutex<AHashMap<SessionId, Arc<Mutex<SessionEntry>>>>>,
}

impl Sessions {
    pub fn create(&self) -> Session {
        let (id, entry) = {
            let mut entries = self.entries.lock().unwrap();

            loop {
                let id = rand::random();

                if let hash_map::Entry::Vacant(entry) = entries.entry(id) {
                    info!(?id, "session created");

                    let entry = entry
                        .insert(Arc::new(Mutex::new(SessionEntry::default())))
                        .clone();

                    break (id, entry);
                }
            }
        };

        let entries = self.entries.clone();

        Session::new(id, entry, move || {
            info!(?id, "session destroyed");
            entries.lock().unwrap().remove(&id);
        })
    }

    pub fn first_id(&self) -> Option<SessionId> {
        self.entries.lock().unwrap().keys().next().cloned()
    }

    pub fn with<T>(
        &self,
        id: SessionId,
        f: impl FnOnce(&mut SessionEntry) -> T,
    ) -> Option<T> {
        self.entries
            .lock()
            .unwrap()
            .get(&id)
            .map(|sess| f(&mut sess.lock().unwrap()))
    }
}

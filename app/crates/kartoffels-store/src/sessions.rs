use crate::{Session, SessionEntry, SessionId};
use kartoffels_utils::Id;
use parking_lot::Mutex;
use std::collections::{hash_map, HashMap};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::info;

#[derive(Debug)]
pub(crate) struct Sessions {
    entries: HashMap<SessionId, Arc<Mutex<SessionEntry>>>,
    abandoned: (mpsc::Sender<SessionId>, mpsc::Receiver<SessionId>),
    test_id: u64,
}

impl Sessions {
    pub fn new() -> Self {
        Self {
            entries: Default::default(),
            abandoned: mpsc::channel(32),
            test_id: 0,
        }
    }

    pub fn create(&mut self, testing: bool) -> Session {
        let (id, entry) = {
            loop {
                let id = SessionId::new(if testing {
                    Id::new({
                        self.test_id += 1;
                        self.test_id
                    })
                } else {
                    rand::random()
                });

                if let hash_map::Entry::Vacant(entry) = self.entries.entry(id) {
                    info!(?id, "session created");

                    let entry = entry
                        .insert(Arc::new(Mutex::new(SessionEntry::new(
                            testing,
                        ))))
                        .clone();

                    break (id, entry);
                }
            }
        };

        Session::new(id, entry).on_abandoned(self.abandoned.0.clone())
    }

    pub fn find(&self, id: impl Into<Option<SessionId>>) -> Vec<Session> {
        if let Some(id) = id.into() {
            self.entries
                .get(&id)
                .into_iter()
                .map(|entry| Session::new(id, entry.clone()))
                .collect()
        } else {
            self.entries
                .iter()
                .map(|(id, entry)| Session::new(*id, entry.clone()))
                .collect()
        }
    }

    pub async fn gc(&mut self) {
        if let Some(id) = self.abandoned.1.recv().await
            && self.entries.remove(&id).is_some()
        {
            info!(?id, "session collected");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SessionRole;
    use kartoffels_utils::Id;

    fn id(id: u64) -> SessionId {
        SessionId::new(Id::new(id))
    }

    #[tokio::test]
    async fn smoke() {
        let mut target = Sessions::new();

        // ---

        let s1 = target.create(true);
        let s2 = target.create(true);
        let s3 = target.create(true);

        assert_eq!(id(1), s1.id());
        assert_eq!(id(2), s2.id());
        assert_eq!(id(3), s3.id());
        assert_eq!(3, target.find(None).len());

        // ---

        assert!(s1.with(|s| s.role().is_user()));
        assert!(s2.with(|s| s.role().is_user()));
        assert!(s3.with(|s| s.role().is_user()));

        target.find(Some(id(2)))[0].with(|s| {
            *s.role_mut() = SessionRole::Admin;
        });

        assert!(s1.with(|s| s.role().is_user()));
        assert!(s2.with(|s| s.role().is_admin()));
        assert!(s3.with(|s| s.role().is_user()));

        // ---

        drop(s1);
        target.gc().await;

        assert_eq!(2, target.find(None).len());
        assert!(target.find(id(1)).is_empty());
        assert!(!target.find(id(2)).is_empty());
        assert!(!target.find(id(3)).is_empty());

        // --

        drop(s3);
        target.gc().await;

        assert_eq!(1, target.find(None).len());
        assert!(target.find(id(1)).is_empty());
        assert!(!target.find(id(2)).is_empty());
        assert!(target.find(id(3)).is_empty());

        // --

        drop(s2);
        target.gc().await;

        assert!(target.find(None).is_empty());
        assert!(target.find(id(1)).is_empty());
        assert!(target.find(id(2)).is_empty());
        assert!(target.find(id(3)).is_empty());
    }
}

use crate::{Session, SessionEntry, SessionId};
use ahash::AHashMap;
use rand::{Rng, RngCore};
use std::collections::hash_map;
use std::sync::{Arc, Mutex};
use tracing::info;

#[derive(Debug, Default)]
pub struct Sessions {
    entries: Arc<Mutex<AHashMap<SessionId, Arc<Mutex<SessionEntry>>>>>,
}

impl Sessions {
    pub fn create(&self, rng: &mut impl RngCore) -> Session {
        let (id, entry) = {
            let mut entries = self.entries.lock().unwrap();

            loop {
                let id = SessionId(rng.gen());

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

#[cfg(test)]
mod tests {
    use super::*;
    use kartoffels_utils::Id;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use std::str::FromStr;

    #[test]
    fn smoke() {
        let target = Sessions::default();
        let mut rng = ChaCha8Rng::from_seed(Default::default());

        let s1 = target.create(&mut rng);
        let s2 = target.create(&mut rng);
        let s3 = target.create(&mut rng);

        assert_eq!(
            SessionId(Id::from_str("d640-5f89-2fef-003e").unwrap()),
            s1.id(),
        );

        assert_eq!(
            SessionId(Id::from_str("a1a5-091f-e8b8-5b7f").unwrap()),
            s2.id(),
        );

        assert_eq!(
            SessionId(Id::from_str("3b7f-9ace-c30e-842c").unwrap()),
            s3.id(),
        );

        // ---

        assert!(!s1.with(|s| s.is_admin()));
        assert!(!s2.with(|s| s.is_admin()));
        assert!(!s3.with(|s| s.is_admin()));

        target.with(s2.id(), |s| s.make_admin());

        assert!(!s1.with(|s| s.is_admin()));
        assert!(s2.with(|s| s.is_admin()));
        assert!(!s3.with(|s| s.is_admin()));

        // ---

        assert!(target.first_id().is_some());
        assert_eq!(Some(false), target.with(s1.id(), |s| s.is_admin()));
        assert_eq!(Some(true), target.with(s2.id(), |s| s.is_admin()));
        assert_eq!(Some(false), target.with(s3.id(), |s| s.is_admin()));

        let s1_id = s1.id();
        let s2_id = s2.id();
        let s3_id = s3.id();

        drop(s1);

        assert!(target.first_id().is_some());
        assert_eq!(None, target.with(s1_id, |s| s.is_admin()));
        assert_eq!(Some(true), target.with(s2_id, |s| s.is_admin()));
        assert_eq!(Some(false), target.with(s3_id, |s| s.is_admin()));

        drop(s3);

        assert!(target.first_id().is_some());
        assert_eq!(None, target.with(s1_id, |s| s.is_admin()));
        assert_eq!(Some(true), target.with(s2_id, |s| s.is_admin()));
        assert_eq!(None, target.with(s3_id, |s| s.is_admin()));

        drop(s2);

        assert!(target.first_id().is_none());
        assert_eq!(None, target.with(s1_id, |s| s.is_admin()));
        assert_eq!(None, target.with(s2_id, |s| s.is_admin()));
        assert_eq!(None, target.with(s3_id, |s| s.is_admin()));
    }
}

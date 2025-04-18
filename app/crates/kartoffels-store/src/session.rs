use derivative::Derivative;
use kartoffels_utils::Id;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Session {
    id: SessionId,
    entry: Arc<Mutex<SessionEntry>>,

    #[derivative(Debug = "ignore")]
    on_drop: Option<Box<dyn FnOnce() + Send + Sync>>,
}

impl Session {
    pub(crate) fn new(
        id: SessionId,
        entry: Arc<Mutex<SessionEntry>>,
        on_drop: impl FnOnce() + Send + Sync + 'static,
    ) -> Self {
        Self {
            id,
            entry,
            on_drop: Some(Box::new(on_drop)),
        }
    }

    pub fn id(&self) -> SessionId {
        self.id
    }

    pub fn with<T>(&self, f: impl FnOnce(&mut SessionEntry) -> T) -> T {
        f(&mut self.entry.lock().unwrap())
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        if let Some(f) = self.on_drop.take() {
            f();
        }
    }
}

#[derive(Debug, Default)]
pub struct SessionEntry {
    role: SessionRole,
    upload: Option<oneshot::Sender<Vec<u8>>>,
}

impl SessionEntry {
    pub fn is_admin(&self) -> bool {
        self.role == SessionRole::Admin
    }

    pub fn make_admin(&mut self) {
        self.role = SessionRole::Admin;
    }

    pub fn request_upload(&mut self) -> SessionUploadInterest {
        let (tx, rx) = oneshot::channel();

        self.upload = Some(tx);

        SessionUploadInterest { rx }
    }

    #[allow(clippy::result_unit_err)]
    pub fn complete_upload(&mut self, src: Vec<u8>) -> Result<(), ()> {
        if let Some(tx) = self.upload.take() {
            _ = tx.send(src);

            Ok(())
        } else {
            Err(())
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SessionRole {
    Admin,

    #[default]
    User,
}

#[derive(Debug)]
pub struct SessionUploadInterest {
    rx: oneshot::Receiver<Vec<u8>>,
}

impl SessionUploadInterest {
    pub fn try_recv(&mut self) -> Option<Vec<u8>> {
        self.rx.try_recv().ok()
    }
}

#[derive(
    Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Derivative,
)]
#[derivative(Debug = "transparent")]
pub struct SessionId(pub(crate) Id);

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

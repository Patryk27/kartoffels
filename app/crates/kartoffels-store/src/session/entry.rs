use super::SessionRole;
use chrono::{DateTime, Utc};
use tokio::sync::oneshot;

#[derive(Debug)]
pub struct SessionEntry {
    role: SessionRole,
    upload: Option<oneshot::Sender<Vec<u8>>>,
    created_at: DateTime<Utc>,
}

impl SessionEntry {
    pub fn new(testing: bool) -> Self {
        Self {
            role: SessionRole::User,
            upload: None,

            created_at: if testing {
                "2018-01-01T12:00:00Z".parse().unwrap()
            } else {
                Utc::now()
            },
        }
    }

    pub fn role(&self) -> SessionRole {
        self.role
    }

    pub fn role_mut(&mut self) -> &mut SessionRole {
        &mut self.role
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
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

#[derive(Debug)]
pub struct SessionUploadInterest {
    rx: oneshot::Receiver<Vec<u8>>,
}

impl SessionUploadInterest {
    pub fn try_recv(&mut self) -> Option<Vec<u8>> {
        self.rx.try_recv().ok()
    }
}

use super::Snapshot;
use crate::Handle;
use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::sync::watch;
use tokio_stream::wrappers::WatchStream;
use tokio_stream::StreamExt;

#[derive(Debug)]
pub struct SnapshotStream {
    stream: WatchStream<Arc<Snapshot>>,
}

impl SnapshotStream {
    pub(crate) fn new(tx: &watch::Sender<Arc<Snapshot>>) -> Self {
        Self {
            stream: WatchStream::new(tx.subscribe()),
        }
    }

    pub async fn next(&mut self) -> Result<Arc<Snapshot>> {
        self.stream.next().await.context(Handle::ERR)
    }
}

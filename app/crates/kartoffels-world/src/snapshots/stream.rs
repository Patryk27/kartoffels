use crate::*;
use tokio_stream::wrappers::WatchStream;

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

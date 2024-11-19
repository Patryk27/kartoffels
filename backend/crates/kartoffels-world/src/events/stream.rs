use super::{Event, EventLetter};
use crate::{BotId, Handle};
use anyhow::{Context, Result};
use tokio::sync::broadcast;
use tokio_stream::wrappers::errors::BroadcastStreamRecvError;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;
use tracing::warn;

#[derive(Debug)]
pub struct EventStream {
    stream: BroadcastStream<EventLetter>,
    pending: Option<EventLetter>,
}

impl EventStream {
    pub(crate) fn new(tx: &broadcast::Sender<EventLetter>) -> Self {
        Self {
            stream: BroadcastStream::new(tx.subscribe()),
            pending: None,
        }
    }

    pub async fn next(&mut self) -> Result<EventLetter> {
        if let Some(event) = self.pending.take() {
            return Ok(event);
        }

        loop {
            let event = self.stream.next().await.context(Handle::ERR)?;

            match event {
                Ok(event) => {
                    return Ok(event);
                }

                Err(BroadcastStreamRecvError::Lagged(_)) => {
                    warn!("event stream lagged");
                }
            }
        }
    }

    pub async fn sync(&mut self, version: u64) -> Result<()> {
        loop {
            let event = self.next().await?;

            if event.version > version {
                self.pending = Some(event);

                return Ok(());
            }
        }
    }

    pub async fn next_spawned_bot(&mut self) -> Result<BotId> {
        loop {
            if let Event::BotSpawned { id } = self.next().await?.event {
                return Ok(id);
            }
        }
    }

    pub async fn next_killed_bot(&mut self) -> Result<BotId> {
        loop {
            if let Event::BotKilled { id } = self.next().await?.event {
                return Ok(id);
            }
        }
    }
}

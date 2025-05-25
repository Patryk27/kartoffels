use super::{Event, EventEnvelope};
use crate::{BotId, Handle, ObjectId};
use anyhow::{Context, Result};
use tokio::sync::broadcast;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::wrappers::errors::BroadcastStreamRecvError;
use tracing::warn;

#[derive(Debug)]
pub struct EventStream {
    stream: BroadcastStream<EventEnvelope>,
    pending: Option<EventEnvelope>,
}

impl EventStream {
    pub(crate) fn new(tx: &broadcast::Sender<EventEnvelope>) -> Self {
        Self {
            stream: BroadcastStream::new(tx.subscribe()),
            pending: None,
        }
    }

    pub async fn next(&mut self) -> Result<EventEnvelope> {
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

    pub async fn next_born_bot(&mut self) -> Result<BotId> {
        loop {
            if let Event::BotBorn { id } = self.next().await?.event {
                return Ok(id);
            }
        }
    }

    pub async fn next_died_bot(&mut self) -> Result<BotId> {
        loop {
            if let Event::BotDied { id, .. } = self.next().await?.event {
                return Ok(id);
            }
        }
    }

    pub async fn next_dropped_object(&mut self) -> Result<ObjectId> {
        loop {
            if let Event::ObjectDropped { id } = self.next().await?.event {
                return Ok(id);
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
}

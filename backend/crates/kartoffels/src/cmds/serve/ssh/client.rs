use super::AppChannel;
use ahash::AHashMap;
use anyhow::{anyhow, Context, Error, Result};
use axum::async_trait;
use kartoffels_store::Store;
use russh::server::{self, Auth, Msg, Session};
use russh::{Channel, ChannelId, Pty};
use russh_keys::key::PublicKey;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use tracing::{info, instrument};

#[derive(Debug)]
pub struct AppClient {
    addr: String,
    store: Arc<Store>,
    shutdown: CancellationToken,
    channels: AHashMap<ChannelId, AppChannel>,
}

impl AppClient {
    #[instrument(skip(store, shutdown))]
    pub fn new(
        addr: String,
        store: Arc<Store>,
        shutdown: CancellationToken,
    ) -> Self {
        info!("connection opened");

        Self {
            addr,
            store,
            shutdown,
            channels: Default::default(),
        }
    }

    fn channel_mut(&mut self, id: ChannelId) -> Result<&mut AppChannel> {
        self.channels
            .get_mut(&id)
            .with_context(|| format!("unknown channel: {}", id))
    }
}

#[async_trait]
impl server::Handler for AppClient {
    type Error = Error;

    #[instrument(
        skip(self, channel),
        fields(addr = ?self.addr, channel = channel.id().to_string())
    )]
    async fn channel_open_session(
        &mut self,
        channel: Channel<Msg>,
        _: &mut Session,
    ) -> Result<bool> {
        let app_channel =
            AppChannel::new(self.store.clone(), self.shutdown.clone());

        let created =
            self.channels.try_insert(channel.id(), app_channel).is_ok();

        info!("channel opened");

        if created {
            Ok(true)
        } else {
            Err(anyhow!(
                "channel `{}` has been already opened",
                channel.id()
            ))
        }
    }

    #[instrument(skip(self), fields(addr = ?self.addr))]
    async fn channel_close(
        &mut self,
        channel: ChannelId,
        _: &mut Session,
    ) -> Result<()> {
        if self.channels.remove(&channel).is_some() {
            info!("channel closed");

            Ok(())
        } else {
            Err(anyhow!("channel `{}` has been already closed", channel))
        }
    }

    async fn auth_publickey(&mut self, _: &str, _: &PublicKey) -> Result<Auth> {
        Ok(Auth::Accept)
    }

    async fn data(
        &mut self,
        id: ChannelId,
        data: &[u8],
        _: &mut Session,
    ) -> Result<()> {
        self.channel_mut(id)?.data(data).await?;

        Ok(())
    }

    async fn pty_request(
        &mut self,
        id: ChannelId,
        _: &str,
        width: u32,
        height: u32,
        _: u32,
        _: u32,
        _: &[(Pty, u32)],
        session: &mut Session,
    ) -> Result<()> {
        self.channel_mut(id)?
            .pty_request(id, width, height, session)
            .await?;

        Ok(())
    }

    async fn window_change_request(
        &mut self,
        id: ChannelId,
        width: u32,
        height: u32,
        _: u32,
        _: u32,
        _: &mut Session,
    ) -> Result<()> {
        self.channel_mut(id)?
            .window_change_request(width, height)
            .await?;

        Ok(())
    }
}

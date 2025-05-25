use super::AppChannel;
use anyhow::{Context, Error, Result, anyhow};
use glam::uvec2;
use kartoffels_store::Store;
use russh::keys::PublicKey;
use russh::server::{Auth, Handler, Msg, Session};
use russh::{Channel, ChannelId, Pty};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio_util::sync::CancellationToken;
use tracing::{Span, info, info_span};

#[derive(Debug)]
pub struct AppClient {
    store: Arc<Store>,
    shutdown: CancellationToken,
    clients: Arc<AtomicUsize>,
    channels: HashMap<ChannelId, AppChannel>,
    span: Span,
}

impl AppClient {
    pub fn new(
        store: Arc<Store>,
        shutdown: CancellationToken,
        clients: Arc<AtomicUsize>,
        addr: String,
    ) -> Self {
        let span = info_span!("ssh", %addr);

        info!(parent: &span, "connection opened");

        clients.fetch_add(1, Ordering::SeqCst);

        Self {
            span,
            store,
            clients,
            shutdown,
            channels: Default::default(),
        }
    }

    fn channel_mut(&mut self, id: ChannelId) -> Result<&mut AppChannel> {
        self.channels
            .get_mut(&id)
            .with_context(|| format!("unknown channel: {id}"))
    }
}

impl Handler for AppClient {
    type Error = Error;

    async fn auth_none(&mut self, _: &str) -> Result<Auth> {
        Ok(Auth::Accept)
    }

    async fn auth_password(&mut self, _: &str, _: &str) -> Result<Auth> {
        Ok(Auth::Accept)
    }

    async fn auth_publickey(&mut self, _: &str, _: &PublicKey) -> Result<Auth> {
        Ok(Auth::Accept)
    }

    async fn channel_open_session(
        &mut self,
        channel: Channel<Msg>,
        _: &mut Session,
    ) -> Result<bool> {
        let app_channel = AppChannel::new(
            channel.id(),
            self.store.clone(),
            self.shutdown.clone(),
            &self.span,
        );

        let created =
            self.channels.try_insert(channel.id(), app_channel).is_ok();

        if created {
            Ok(true)
        } else {
            Err(anyhow!(
                "channel `{}` has been already opened",
                channel.id()
            ))
        }
    }

    async fn channel_close(
        &mut self,
        channel: ChannelId,
        _: &mut Session,
    ) -> Result<()> {
        if self.channels.remove(&channel).is_some() {
            Ok(())
        } else {
            Err(anyhow!("channel `{}` has been already closed", channel))
        }
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
            .pty_request(id, uvec2(width, height), session)
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
            .window_change_request(uvec2(width, height))
            .await?;

        Ok(())
    }
}

impl Drop for AppClient {
    fn drop(&mut self) {
        info!(parent: &self.span, "connection closed");

        self.clients.fetch_sub(1, Ordering::SeqCst);
    }
}

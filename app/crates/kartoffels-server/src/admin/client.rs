use super::cmds::CmdContext;
use super::{Admins, Cmd};
use anyhow::{Context, Error, Result, anyhow};
use clap::Parser;
use kartoffels_store::Store;
use kartoffels_utils::ErrorExt;
use russh::keys::PublicKey;
use russh::server::{Auth, Handler, Msg, Session};
use russh::{Channel, ChannelId, CryptoVec, Pty};
use std::iter;
use std::sync::Arc;
use tracing::{Span, info, info_span, warn};

#[derive(Debug)]
pub struct AppClient {
    admins: Arc<Admins>,
    store: Arc<Store>,
    auth: bool,
    span: Span,
}

impl AppClient {
    pub fn new(admins: Arc<Admins>, store: Arc<Store>, addr: String) -> Self {
        let span = info_span!("ssh", %addr);

        info!(parent: &span, "connection opened");

        Self {
            admins,
            store,
            auth: false,
            span,
        }
    }
}

impl Handler for AppClient {
    type Error = Error;

    async fn auth_none(&mut self, _: &str) -> Result<Auth> {
        if self.admins.is_none() {
            info!("accepting none-auth");

            self.auth = true;

            Ok(Auth::Accept)
        } else {
            warn!("rejecting none-auth");
            warn!("someone foreign just tried to login to admin panel");

            Ok(Auth::reject())
        }
    }

    async fn auth_publickey(
        &mut self,
        _: &str,
        public_key: &PublicKey,
    ) -> Result<Auth> {
        let key = public_key.fingerprint(Default::default()).to_string();

        if self
            .admins
            .as_ref()
            .as_ref()
            .is_none_or(|admins| admins.contains(public_key.key_data()))
        {
            info!(?key, "accepting key-auth");

            self.auth = true;

            Ok(Auth::Accept)
        } else {
            warn!(?key, "rejecting key-auth");
            warn!(?key, "someone foreign just tried to login to admin panel");

            Ok(Auth::reject())
        }
    }

    async fn channel_open_session(
        &mut self,
        _: Channel<Msg>,
        _: &mut Session,
    ) -> Result<bool> {
        Ok(true)
    }

    async fn data(
        &mut self,
        _: ChannelId,
        _: &[u8],
        _: &mut Session,
    ) -> Result<()> {
        Err(anyhow!("unsupported operation"))
    }

    async fn extended_data(
        &mut self,
        _: ChannelId,
        _: u32,
        _: &[u8],
        _: &mut Session,
    ) -> Result<()> {
        Err(anyhow!("unsupported operation"))
    }

    async fn pty_request(
        &mut self,
        _: ChannelId,
        _: &str,
        _: u32,
        _: u32,
        _: u32,
        _: u32,
        _: &[(Pty, u32)],
        _: &mut Session,
    ) -> Result<()> {
        Err(anyhow!("unsupported operation"))
    }

    async fn exec_request(
        &mut self,
        chan: ChannelId,
        data: &[u8],
        sess: &mut Session,
    ) -> Result<()> {
        assert!(self.auth);

        let mut exit_code = 1;

        let result: Result<()> = try {
            let cmd = String::from_utf8(data.to_vec())?;

            info!(parent: &self.span, ?cmd, "executing");

            let cmd =
                shellwords::split(&cmd).context("couldn't parse command")?;

            let cmd = iter::once("kartoffels".into()).chain(cmd);

            let cmd = Cmd::try_parse_from(cmd).map_err(|err| {
                // Passthrough the error code so that we don't return code=1
                // when user just asks for `--help` etc.
                exit_code = err.exit_code();

                anyhow!("{}", err.to_string().to_lowercase())
            })?;

            cmd.run(&mut CmdContext {
                store: &self.store,
                chan,
                sess,
            })
            .await?
        };

        match result {
            Ok(_) => {
                sess.exit_status_request(chan, 0)?;
                sess.close(chan)?;
            }

            Err(err) => {
                sess.data(
                    chan,
                    CryptoVec::from(err.to_fmt_string().to_lowercase()),
                )?;

                sess.exit_status_request(chan, exit_code as u32)?;
                sess.close(chan)?;
            }
        }

        Ok(())
    }
}

impl Drop for AppClient {
    fn drop(&mut self) {
        info!(parent: &self.span, "connection closed");
    }
}

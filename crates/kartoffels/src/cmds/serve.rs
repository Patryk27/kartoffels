mod http;
mod ssh;

use anyhow::{Context, Result};
use clap::Parser;
use indoc::indoc;
use kartoffels_store::Store;
use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::try_join;
use tracing::info;
use tracing_subscriber::fmt;

const LOGO: &str = indoc! {r#"
     _              _         __  __     _
    | |            | |       / _|/ _|   | |
    | | ____ _ _ __| |_ ___ | |_| |_ ___| |___
    | |/ / _` | '__| __/ _ \|  _|  _/ _ \ / __|
    |   < (_| | |  | || (_) | | | ||  __/ \__ \
    |_|\_\__,_|_|   \__\___/|_| |_| \___|_|___/
"#};

#[derive(Debug, Parser)]
pub struct ServeCmd {
    data: PathBuf,

    #[clap(long)]
    http: Option<SocketAddr>,

    #[clap(long)]
    ssh: Option<SocketAddr>,

    #[clap(long)]
    debug: bool,

    #[clap(long)]
    quiet: bool,
}

impl ServeCmd {
    pub fn run(self) -> Result<()> {
        let filter = env::var("RUST_LOG").unwrap_or_else(|_| {
            let filter = if self.debug {
                "tower_http=debug,kartoffels=debug"
            } else {
                "kartoffels=info"
            };

            filter.to_owned()
        });

        if self.quiet {
            println!("starting");
        } else {
            tracing_subscriber::fmt()
                .with_env_filter(filter)
                .event_format(fmt::format::Format::default().without_time())
                .init();
        }

        for line in LOGO.lines() {
            info!("{}", line);
        }

        info!("");
        info!(?self, "starting");

        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?
            .block_on(async move {
                let store =
                    Store::open(&self.data).await.with_context(|| {
                        format!(
                            "couldn't load store from `{}`",
                            self.data.display()
                        )
                    })?;

                let store = Arc::new(store);

                let http = {
                    let store = store.clone();

                    async {
                        if let Some(addr) = &self.http {
                            http::start(addr, store).await
                        } else {
                            Ok(())
                        }
                    }
                };

                let ssh = async {
                    if let Some(addr) = &self.ssh {
                        ssh::start(addr, store).await
                    } else {
                        Ok(())
                    }
                };

                info!("ready");

                try_join!(http, ssh)?;

                Ok(())
            })
    }
}

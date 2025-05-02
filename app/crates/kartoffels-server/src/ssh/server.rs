use super::AppClient;
use kartoffels_store::Store;
use russh::server::Server;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub struct AppServer {
    store: Arc<Store>,
    shutdown: CancellationToken,
}

impl AppServer {
    pub fn new(store: Arc<Store>, shutdown: CancellationToken) -> Self {
        Self { store, shutdown }
    }
}

impl Server for AppServer {
    type Handler = AppClient;

    fn new_client(&mut self, addr: Option<SocketAddr>) -> AppClient {
        let addr = addr
            .map(|addr| addr.to_string())
            .unwrap_or_else(|| "-".into());

        AppClient::new(self.store.clone(), self.shutdown.clone(), addr)
    }
}

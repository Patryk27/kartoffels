use super::AppClient;
use kartoffels_store::Store;
use russh::server;
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Debug)]
pub struct AppServer {
    store: Arc<Store>,
}

impl AppServer {
    pub fn new(store: Arc<Store>) -> Self {
        Self { store }
    }
}

// TODO consider some ddos / rate limiting mechanism
impl server::Server for AppServer {
    type Handler = AppClient;

    fn new_client(&mut self, addr: Option<SocketAddr>) -> AppClient {
        let addr = addr
            .map(|addr| addr.to_string())
            .unwrap_or_else(|| "-".into());

        AppClient::new(addr, self.store.clone())
    }
}

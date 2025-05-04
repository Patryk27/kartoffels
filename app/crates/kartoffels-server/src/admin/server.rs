use super::{Admins, AppClient};
use kartoffels_store::Store;
use russh::server::Server;
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Debug)]
pub struct AppServer {
    admins: Arc<Admins>,
    store: Arc<Store>,
}

impl AppServer {
    pub fn new(admins: Admins, store: Arc<Store>) -> Self {
        Self {
            admins: Arc::new(admins),
            store,
        }
    }
}

impl Server for AppServer {
    type Handler = AppClient;

    fn new_client(&mut self, addr: Option<SocketAddr>) -> AppClient {
        let addr = addr
            .map(|addr| addr.to_string())
            .unwrap_or_else(|| "-".into());

        AppClient::new(self.admins.clone(), self.store.clone(), addr)
    }
}

use anyhow::{Ok, Result};
use dashmap::DashMap;
use std::sync::Arc;
use tracing::info;

use crate::{
    tcp_server::{
        control_server::ControlServer, event_server::EventServer, http_server::HttpServer,
        tcp_listener::TcpServer,
    },
    tunnel::Tunnel,
};
pub(crate) type ServerContext = DashMap<String, Tunnel>;

pub struct Server {
    domain: String,
    http_port: u16,
    server_context: Arc<ServerContext>,
}
impl Server {
    pub async fn new(domain: String, http_port: u16) -> Server {
        Server {
            domain,
            server_context: Arc::new(ServerContext::default()),
            http_port,
        }
    }

    // Start the server, listening for new connections.
    fn listen<S: TcpServer + 'static>(&self, event_server: S) {
        let context = self.server_context.clone();
        tokio::spawn(async move {
            event_server.listen(context).await.unwrap();
            info!("exiting listener");
        });
    }

    pub async fn start(self) -> Result<()> {
        self.listen(ControlServer::new(self.domain.clone()).await?);
        self.listen(HttpServer::new(self.http_port).await?);
        self.listen(EventServer::new().await?);
        Ok(())
    }
}

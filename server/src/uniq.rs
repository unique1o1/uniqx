use anyhow::{Ok, Result};
use dashmap::DashMap;
use std::sync::Arc;
use tracing::info;

use crate::{
    tcp_server::{
        control_server::ControlServer, event_server::EventServer, http_server::PublicHttpServer,
        tcp_listener::TcpServer,
    },
    tunnel::Tunnel,
};
pub(crate) type ServerContext = DashMap<String, Tunnel>;

pub struct Server {
    domain: String,
    server_context: Arc<ServerContext>,
}
impl Server {
    pub async fn new(domain: String) -> Server {
        Server {
            domain: domain,
            server_context: Arc::new(ServerContext::default()),
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
        self.listen(PublicHttpServer::new().await?);
        self.listen(EventServer::new().await?);
        // this.http_listen();
        Ok(())
    }
}

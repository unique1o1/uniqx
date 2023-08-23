use anyhow::{Ok, Result};
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
    tcp_server::{
        event_server::EventServer, public_control_server::ControlServer,
        public_http_server::PublicHttpServer, tcp_listener::TcpServer,
    },
    tunnel::Tunnel,
};
pub(crate) type ServerContext = DashMap<String, Tunnel>;

pub struct Server {
    server_context: Arc<ServerContext>,
}
impl Server {
    pub async fn new() -> Server {
        Server {
            server_context: Arc::new(ServerContext::default()),
        }
    }

    // Start the server, listening for new connections.
    fn listen<S: TcpServer + 'static>(&self, event_server: S) {
        let context = self.server_context.clone();
        tokio::spawn(async move {
            event_server.listen(context).await.unwrap();
            println!("exiting listener");
        });
    }

    pub async fn start(self) -> Result<()> {
        self.listen(ControlServer::new().await?);
        self.listen(PublicHttpServer::new().await?);
        self.listen(EventServer::new().await?);
        // this.http_listen();
        Ok(())
    }
}

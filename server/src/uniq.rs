use crate::{
    event_server::{EventServer, Tunnel},
    http_server::HttpServer,
};
use anyhow::{Ok, Result};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use tracing::event;

#[derive(Default)]
pub struct ServerContext {
    pub http_tunnels: HashMap<String, Tunnel>, // client_conn: TcpListener,
}
pub struct Server {
    domain: String,

    server_context: Arc<Mutex<ServerContext>>,
    // tunnel: Tunnel,
    // httpServer: HttpServer,
}
impl Server {
    pub async fn new(domain: String) -> Server {
        Server {
            domain: domain,
            server_context: Arc::new(Mutex::new(ServerContext::default())),
        }
    }
    fn http_listen(self, http_server: HttpServer) {
        let context: Arc<Mutex<ServerContext>> = self.server_context.clone();
        tokio::spawn(async move {
            http_server.listen(context).await.unwrap();
        });
    }

    fn public_http_listen(self, http_server: HttpServer) {
        let context = self.server_context.clone();
        tokio::spawn(async move {
            http_server.listen(context).await.unwrap();
        });
    }
    // /// Start the server, listening for new connections.
    fn listen(&self, event_server: EventServer) {
        let context = self.server_context.clone();
        tokio::spawn(async move {
            event_server.listen(context).await.unwrap();
        });
    }

    pub async fn start(self) -> Result<()> {
        self.listen(EventServer::new().await);
        self.http_listen(HttpServer::new().await);
        // this.http_listen();
        Ok(())
    }
}

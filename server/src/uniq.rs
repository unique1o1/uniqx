use anyhow::{Ok, Result};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{Mutex, RwLock};

use crate::{
    tcp_server::{
        event_server::EventServer, http_event_server::HttpEventServer, http_server::HttpServer,
        tcp_listener::TcpServer,
    },
    tunnel::Tunnel,
};
pub(crate) type ServerContext = HashMap<String, Mutex<Tunnel>>;

pub struct Server {
    domain: String,

    server_context: Arc<RwLock<ServerContext>>,
    // tunnel: Tunnel,
    // httpServer: HttpServer,
}
impl Server {
    pub async fn new(domain: String) -> Server {
        Server {
            domain: domain,
            server_context: Arc::new(RwLock::new(ServerContext::default())),
        }
    }
    // fn http_listen(self, http_server: HttpServer) {
    //     let context: Arc<RwLock<ServerContext>> = self.server_context.clone();
    //     tokio::spawn(async move {
    //         http_server.listen(context).await.unwrap();
    //     });
    // }

    // fn public_http_listen(self, http_server: HttpServer) {
    //     let context = self.server_context.clone();
    //     tokio::spawn(async move {
    //         http_server.listen(context).await.unwrap();
    //     });
    // }
    // Start the server, listening for new connections.
    fn listen<S: TcpServer + 'static>(&self, event_server: S) {
        let context = self.server_context.clone();
        tokio::spawn(async move {
            event_server.listen(context).await.unwrap();
        });
    }

    pub async fn start(self) -> Result<()> {
        self.listen(EventServer::new().await);
        self.listen(HttpServer::new().await);
        self.listen(HttpEventServer::new().await);
        // this.http_listen();
        Ok(())
    }
}

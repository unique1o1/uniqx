use async_trait::async_trait;
use std::sync::Arc;

use anyhow::Result;
use shared::HTTP_EVENT_SERVER_PORT;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{Mutex, RwLock},
};

use crate::uniq::ServerContext;

use super::tcp_listener::{EventHandler, TCPListener, TcpServer};

pub struct HttpEventServer {
    listener: TcpListener,
}

impl HttpEventServer {
    pub async fn new() -> Self {
        let listener = TcpListener::bind(("0.0.0.0", HTTP_EVENT_SERVER_PORT))
            .await
            .unwrap();
        Self {
            // listener: Arc::new(Mutex::new(listener)),
            listener: listener,
        }
    }
}
impl TCPListener for HttpEventServer {
    #[inline]
    fn listener(&self) -> &TcpListener {
        return &self.listener;
    }
}

#[async_trait]
impl EventHandler for HttpEventServer {
    async fn handle_conn(
        &self,
        mut stream: TcpStream,
        context: Arc<RwLock<ServerContext>>,
    ) -> Result<()> {
        todo!()
    }
}
impl TcpServer for HttpEventServer {}

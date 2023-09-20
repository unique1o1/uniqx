use anyhow::{Context, Result};
use async_trait::async_trait;
use shared::{delimited::DelimitedWriteExt, structs::NewClient};

use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tracing::info;
use uuid::Uuid;

use crate::uniqx::ServerContext;

use super::tcp_listener::{EventHandler, TCPListener};
pub struct TcpServer {
    listener: TcpListener,
}

impl TcpServer {
    pub async fn new(port: u16) -> Self {
        let listener = TcpListener::bind(("0.0.0.0", port)).await.unwrap();

        Self { listener }
    }
}
#[async_trait]
impl EventHandler for TcpServer {
    async fn handle_conn(&self, stream: TcpStream, context: Arc<ServerContext>) -> Result<()> {
        let identifier = Uuid::new_v4().to_string();

        let port = stream.local_addr().unwrap().port();
        info!(?port, "new tcp connection");
        let t = context.get(&port.to_string()).unwrap();
        t.public_conn.insert(identifier.clone(), stream);
        t.control_conn
            .lock()
            .await
            .send_delimited(NewClient {
                initial_buffer: None,
                public_conn_identifier: identifier.clone(),
                control_server_identifier: Some(port.to_string()),
            })
            .await
            .context("error while sending new client info to client")?;

        Ok(())
    }
}

impl TCPListener for TcpServer {
    fn listener(&self) -> &TcpListener {
        &self.listener
    }
}

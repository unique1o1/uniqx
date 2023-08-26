use anyhow::{Context, Error, Result};
use async_trait::async_trait;
use shared::{delimited::DelimitedWriteExt, structs::NewClient};

use std::sync::Arc;
use tokio::{
    io::AsyncReadExt,
    net::{TcpListener, TcpStream},
};
use tracing::info;
use uuid::Uuid;

use crate::uniqx::ServerContext;

use super::tcp_listener::{EventHandler, TCPListener};
pub struct PublicTcpServer {
    listener: TcpListener,
}

async fn parse_host(mut r: impl AsyncReadExt + Unpin) -> Result<(String, Vec<u8>)> {
    let mut buffer = vec![0; 2048];
    let size = r.read(&mut buffer).await?;
    let buffer = &buffer[..size];
    let text = String::from_utf8_lossy(buffer);
    let left = text.find("Host: ").ok_or(Error::msg("no host detected"))? + 6;
    let text = &text[left..];
    let right = text.find('\n').ok_or(Error::msg("no host detected"))?;
    let host = text[..right].trim().to_owned();
    let subdomain = host.split('.').next().unwrap().to_owned();
    Ok((subdomain, buffer.to_owned()))
}
impl PublicTcpServer {
    pub async fn new(port: u16) -> Self {
        let listener = TcpListener::bind(("0.0.0.0", port)).await.unwrap();

        Self {
            // listener: Arc::new(Mutex::new(listener)),
            listener: listener,
        }
    }
}
#[async_trait]
impl EventHandler for PublicTcpServer {
    async fn handle_conn(&self, stream: TcpStream, context: Arc<ServerContext>) -> Result<()> {
        let identifier = Uuid::new_v4().to_string();

        let port = stream.local_addr().unwrap().port();
        info!(?port, "new tcp connection");
        let t = context.get(&port.to_string()).unwrap();
        t.public_conn.insert(identifier.clone(), stream);
        t.event_conn
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

impl TCPListener for PublicTcpServer {
    fn listener(&self) -> &TcpListener {
        &self.listener
    }
}

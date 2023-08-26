use async_trait::async_trait;
use std::sync::Arc;
use tracing::info;

use anyhow::{Error, Result};
use shared::{
    delimited::{delimited_framed, DelimitedReadExt},
    structs::NewClient,
    utils::proxy,
    EVENT_SERVER_PORT,
};
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
};

use crate::uniqx::ServerContext;

use super::tcp_listener::{EventHandler, TCPListener};

pub struct EventServer {
    listener: TcpListener,
}

impl EventServer {
    pub async fn new() -> Result<Self> {
        let listener = TcpListener::bind(("0.0.0.0", EVENT_SERVER_PORT))
            .await
            .unwrap();
        Ok(Self { listener: listener })
    }
}
impl TCPListener for EventServer {
    #[inline]
    fn listener(&self) -> &TcpListener {
        return &self.listener;
    }
}

#[async_trait]
impl EventHandler for EventServer {
    async fn handle_conn(&self, mut stream: TcpStream, context: Arc<ServerContext>) -> Result<()> {
        let data: NewClient = delimited_framed(&mut stream).recv_delimited().await?;
        let t = match context.get(&data.control_server_identifier.unwrap()) {
            Some(t) => t,
            None => {
                return Err(Error::msg("tunnel not found"));
            }
        };
        let (_, public_http_conn) = t.public_conn.remove(&data.public_conn_identifier).unwrap();
        drop(t);
        if data.initial_buffer.is_some() {
            stream.write_all(&data.initial_buffer.unwrap()).await?;
        }
        proxy(public_http_conn, stream).await?;

        Ok(())
    }
}
// impl TcpServer for HttpEventServer {}

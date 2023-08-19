use async_trait::async_trait;
use std::sync::Arc;
use tracing::info;

use anyhow::{Error, Result};
use shared::{frame::Delimited, structs::NewClient, utils::bind, HTTP_EVENT_SERVER_PORT};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf},
    net::{TcpListener, TcpStream},
};

use crate::uniq::ServerContext;

use super::{
    http_server,
    tcp_listener::{EventHandler, TCPListener},
};

pub struct HttpEventServer {
    listener: TcpListener,
}

impl HttpEventServer {
    pub async fn new() -> Self {
        let listener = TcpListener::bind(("0.0.0.0", HTTP_EVENT_SERVER_PORT))
            .await
            .unwrap();
        Self { listener: listener }
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
    async fn handle_conn(&self, mut stream: TcpStream, context: Arc<ServerContext>) -> Result<()> {
        info!("=======incoming http event connection======");
        let data: NewClient = Delimited::new(&mut stream).recv().await?;
        let t = match context.get(&data.subdomain) {
            Some(t) => t,
            None => {
                return Err(Error::msg("tunnel not found"));
            }
        };
        let (_, public_http_conn) = t.public_http_conn.remove(&data.identifier).unwrap();
        let buffer = t.initialBuffer.get(&data.identifier).unwrap();
        println!("length: {}", buffer.len());
        stream.write_all(&buffer).await?;
        let (s1_read, s1_write) = io::split(stream);
        let (s2_read, s2_write) = io::split(public_http_conn);
        tokio::spawn(async move { bind(s1_read, s2_write).await.unwrap() });
        bind(s2_read, s1_write).await.unwrap();
        println!("================http event connection exited===============");
        Ok(())
    }
}
// impl TcpServer for HttpEventServer {}

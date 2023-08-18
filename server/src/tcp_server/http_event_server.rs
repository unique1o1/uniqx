use async_trait::async_trait;
use std::sync::Arc;
use tracing::info;

use anyhow::{Error, Result};
use shared::{frame::Delimited, structs::NewClient, HTTP_EVENT_SERVER_PORT};
use tokio::{
    io::{self, AsyncWriteExt},
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
        info!("incoming http event connection");
        let data: NewClient = Delimited::new(&mut stream).recv().await?;
        let t = match context.get(&data.subdomain) {
            Some(t) => t,
            None => {
                return Err(Error::msg("tunnel not found"));
            }
        };
        let t = t.lock().await;
        let (_, public_http_conn) = t.public_http_conn.remove(&data.identifier).unwrap();

        let buffer = t.initialBuffer.get(&data.identifier).unwrap();
        stream.write_all(&buffer).await?;
        let (mut s1_read, mut s1_write) = io::split(stream);
        let (mut s2_read, mut s2_write) = io::split(public_http_conn);
        loop {
            tokio::select! {
                res = io::copy(&mut s1_read, &mut s2_write) => res,
                res = io::copy(&mut s2_read, &mut s1_write) => res,
            }?;
        }
    }
}
// impl TcpServer for HttpEventServer {}

use async_trait::async_trait;
use std::sync::Arc;

use anyhow::{Error, Result};
use shared::{frame::Delimited, structs::NewClient, utils::proxy, HTTP_EVENT_SERVER_PORT};
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
};

use crate::uniq::ServerContext;

use super::tcp_listener::{EventHandler, TCPListener};

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
        let data: NewClient = Delimited::new(&mut stream).recv().await?;
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
        proxy(stream, public_http_conn).await?;
        // let (s1_read, s1_write) = io::split(stream);
        // let (s2_read, s2_write) = io::split(public_http_conn);
        // let mut set = JoinSet::new();

        // set.spawn(async move { bind(s1_read, s2_write).await.context("cant read from s1") });

        // set.spawn(async move { bind(s2_read, s1_write).await.context("cant read from s2") });
        // set.join_next().await;
        // set.abort_all();
        Ok(())
    }
}
// impl TcpServer for HttpEventServer {}

use anyhow::{Error, Result};
use async_trait::async_trait;
use shared::{structs::NewClient, utils::write_response};
use std::{collections::HashMap, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::{Mutex, RwLock},
};
use tracing::info;
use uuid::{uuid, Uuid};

use crate::uniq::ServerContext;

use super::tcp_listener::{EventHandler, TCPListener, TcpServer};
pub struct HttpServer {
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
impl HttpServer {
    pub async fn new() -> Self {
        let listener = TcpListener::bind(("0.0.0.0", 8001)).await.unwrap();
        Self {
            // listener: Arc::new(Mutex::new(listener)),
            listener: listener,
        }
    }
}
#[async_trait]
impl EventHandler for HttpServer {
    async fn handle_conn(
        &self,
        mut stream: TcpStream,
        context: Arc<RwLock<ServerContext>>,
    ) -> Result<()> {
        let identifier = Uuid::new_v4().to_string();
        let Ok(( subdomain, buffer )) = parse_host(&mut stream).await else{
                write_response(stream, 400,"Bad Request", "Bad Request").await?;
                return Err(Error::msg("parse host error"));
        };
        println!("subdomain: {}", subdomain);
        // stream.write(b"HTTP/1.1 200 Success\r\n\r\n").await.unwrap();
        let context_lock = context.read().await;
        let t = if let Some(t) = context_lock.get(&subdomain) {
            t
        } else {
            write_response(stream, 404, "Not Found", "tunnel not found").await?;
            return Ok(());
        };
        let t = t.lock().await;

        t.public_http_conn
            .lock()
            .await
            .insert(identifier.clone(), stream);
        t.event_conn
            .clone()
            .unwrap()
            .lock()
            .await
            .send(NewClient {
                identifier: identifier,
                // data: buffer,
            })
            .await?;
        info!("new client: ---",);
        // tunnel.public_http_conn = Some(Arc::new(Mutex::new(stream)));
        // context.write().await.insert(subdomain.clone(), tunnel);
        Ok(())
    }
}

impl TCPListener for HttpServer {
    fn listener(&self) -> &TcpListener {
        &self.listener
    }
}
impl TcpServer for HttpServer {}

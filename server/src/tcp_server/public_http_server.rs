use anyhow::{Context, Error, Result};
use async_trait::async_trait;
use shared::{delimited::DelimitedWriteExt, structs::NewClient, utils::write_response};
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
pub struct PublicHttpServer {
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
impl PublicHttpServer {
    pub async fn new() -> Result<Self> {
        let listener = TcpListener::bind(("0.0.0.0", 8009)).await?;
        Ok(Self {
            // listener: Arc::new(Mutex::new(listener)),
            listener: listener,
        })
    }
}
#[async_trait]
impl EventHandler for PublicHttpServer {
    async fn handle_conn(&self, mut stream: TcpStream, context: Arc<ServerContext>) -> Result<()> {
        let identifier = Uuid::new_v4().to_string();
        let Ok((subdomain, buffer)) = parse_host(&mut stream).await else {
            write_response(stream, 400, "Bad Request", "Bad Request").await?;
            return Err(Error::msg("parse host error"));
        };
        let t = match context.get(&subdomain) {
            Some(t) => t,
            None => {
                write_response(stream, 404, "Not Found", "tunnel not found").await?;
                return Ok(());
            }
        };
        t.public_conn.insert(identifier.clone(), stream);
        t.event_conn
            .lock()
            .await
            .send_delimited(NewClient {
                initial_buffer: Some(buffer),
                public_conn_identifier: identifier.clone(),
                control_server_identifier: Some(subdomain),
            })
            .await
            .context("error while sending new client info to client")?;

        Ok(())
    }
}

impl TCPListener for PublicHttpServer {
    fn listener(&self) -> &TcpListener {
        &self.listener
    }
}

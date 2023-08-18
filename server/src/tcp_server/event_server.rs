use anyhow::Result;
use async_trait::async_trait;
use shared::frame::{DelimitedRead, DelimitedWrite};
use shared::utils::DeferCall;
use shared::{
    defer,
    structs::{Protocol, TunnelOpen, TunnelRequest},
    utils::validate_subdomain,
    SERVER_PORT,
};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};

use crate::tunnel::Tunnel;
use crate::uniq::ServerContext;

use super::tcp_listener::{EventHandler, TCPListener};

pub struct EventServer {
    listener: TcpListener,
}
impl TCPListener for EventServer {
    #[inline]
    fn listener(&self) -> &TcpListener {
        return &self.listener;
    }
}
#[async_trait]
impl EventHandler for EventServer {
    async fn handle_conn(&self, stream: TcpStream, context: Arc<ServerContext>) -> Result<()> {
        // let mut inner = self.0.into_inner();
        let (a, b) = tokio::io::split(stream);
        let (mut read, mut write) = (DelimitedRead::new(a), DelimitedWrite::new(b));

        let data: TunnelRequest = read.recv().await?;
        if let Err(msg) = validate_subdomain(&data.subdomain) {
            let data: TunnelOpen = TunnelOpen::with_error(msg);
            write.send(data).await?;
        }

        match data.protocol {
            Protocol::HTTP => {
                write
                    .send(TunnelOpen {
                        error_message: None,
                    })
                    .await?;

                context.insert(
                    data.subdomain.clone(),
                    Tunnel::with_event_conn(write).into(),
                );
                defer! {
                    context
                   .remove(&data.subdomain)
                }
                loop {
                    read.recv().await?;
                }
            }
            Protocol::TCP => {
                todo!()
            }
        }
    }
}

impl EventServer {
    pub async fn new() -> Self {
        let listener = TcpListener::bind(("0.0.0.0", SERVER_PORT)).await.unwrap();
        Self { listener: listener }
    }
}
// impl TcpServer for EventServer {}

use anyhow::{Context, Result};
use async_trait::async_trait;
use futures::StreamExt;
use shared::delimited::{
    delimited_framed_read, delimited_framed_write, DelimitedReadExt, DelimitedWriteExt,
};
use shared::structs::NewClient;
use shared::utils::DeferCall;
use shared::{
    defer,
    structs::{TunnelOpen, TunnelRequest},
    utils::validate_subdomain,
    SERVER_PORT,
};
use shared::{Protocol, NETWORK_TIMEOUT, TCP_KEEPCNT, TCP_KEEPIDLE, TCP_KEEPINTVL};
use socket2::{SockRef, TcpKeepalive};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tracing::info;

use crate::tcp_server::tcp_server::PublicTcpServer;
use crate::tunnel::Tunnel;
use crate::uniq::ServerContext;

use super::tcp_listener::{EventHandler, EventListener, TCPListener};

pub struct ControlServer {
    domain: String,
    listener: TcpListener,
}
impl TCPListener for ControlServer {
    #[inline]
    fn listener(&self) -> &TcpListener {
        return &self.listener;
    }
}
#[async_trait]
impl EventHandler for ControlServer {
    async fn handle_conn(&self, stream: TcpStream, context: Arc<ServerContext>) -> Result<()> {
        let (a, b) = tokio::io::split(stream);

        let (mut read, mut write) = (delimited_framed_read(a), delimited_framed_write(b));
        info!("new client");
        let data: TunnelRequest = read.recv_delimited().await?;

        match data.protocol {
            Protocol::HTTP => {
                if let Err(msg) = validate_subdomain(&data.subdomain) {
                    let data: TunnelOpen = TunnelOpen::with_error(&msg);
                    write.send_delimited(data).await?;
                }
                if context.contains_key(&data.subdomain) {
                    let data: TunnelOpen =
                        TunnelOpen::with_error("subdomain already in use".into());
                    write.send_delimited(data).await?;
                    return Ok(());
                }
                write
                    .send_delimited(TunnelOpen {
                        access_point: format!("{}.{}", &data.subdomain, &self.domain),
                        ..Default::default()
                    })
                    .await?;
                context.insert(
                    data.subdomain.clone(),
                    Tunnel::with_event_conn(write).into(),
                );
                defer! {
                    context
                   .remove(&data.subdomain.clone())
                }

                let _: NewClient = read.recv_delimited().await.context("client disconnected")?;

                Ok(())
            }
            Protocol::TCP => {
                // let listener = TcpListener::bind(("0.0.0.0", data.tcp_port.unwrap())).await?;
                if context.contains_key(&data.tcp_port.unwrap().to_string()) {
                    write
                        .send_delimited(TunnelOpen::with_error("Port already in use"))
                        .await?;
                    return Ok(());
                }
                let listener = PublicTcpServer::new(data.tcp_port.unwrap()).await;
                let context_clone = context.clone();
                // let mut set = JoinSet::new();

                let handle = tokio::spawn(async move {
                    listener.listen(context_clone).await.unwrap();
                    info!("exiting listener");
                });

                write
                    .send_delimited(TunnelOpen {
                        access_point: format!("{}.{}", &data.subdomain, &self.domain),
                        ..Default::default()
                    })
                    .await?;
                context.insert(
                    data.tcp_port.unwrap().to_string(),
                    Tunnel::with_event_conn(write).into(),
                );
                defer! {
                    context
                   .remove(

                                       & data.tcp_port.unwrap().to_string(),
                   )
                }
                // let handle = Arc::new(handle);

                let _: Result<NewClient> =
                    read.recv_delimited().await.context("client disconnected");
                handle.abort();
                Ok(())
            }
        }
    }
}

impl ControlServer {
    pub async fn new(domain: String) -> Result<Self> {
        let addr = SocketAddr::from(([0, 0, 0, 0], SERVER_PORT));
        let listener = TcpListener::bind(addr).await?;
        let keepalive = TcpKeepalive::new()
            .with_time(Duration::from_secs(TCP_KEEPIDLE))
            .with_interval(Duration::from_secs(TCP_KEEPINTVL))
            .with_retries(TCP_KEEPCNT);
        SockRef::from(&listener)
            .set_tcp_keepalive(&keepalive)
            .unwrap();
        info!(?addr, "server listening");
        Ok(Self { listener, domain })
    }
}

use anyhow::{Context, Ok, Result};
use async_trait::async_trait;
use shared::delimited::{
    delimited_framed_read, delimited_framed_write, DelimitedReadExt, DelimitedWriteExt,
};
use shared::structs::NewClient;
use shared::utils::DeferCall;
use shared::NETWORK_TIMEOUT;
use shared::{
    defer,
    structs::{Protocol, TunnelOpen, TunnelRequest},
    utils::validate_subdomain,
    SERVER_PORT,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinSet;
use tokio::time::timeout;
use tokio_util::codec::FramedRead;
use tracing::info;

use crate::tcp_server::public_tcp_server::PublicTcpServer;
use crate::tunnel::Tunnel;
use crate::uniq::ServerContext;

use super::tcp_listener::{EventHandler, EventListener, TCPListener};

pub struct ControlServer {
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
                if let Err(msg) = validate_subdomain(data.subdomain.as_ref().unwrap()) {
                    let data: TunnelOpen = TunnelOpen::with_error(&msg);
                    write.send_delimited(data).await?;
                }
                if context.contains_key(data.subdomain.as_ref().unwrap()) {
                    let data: TunnelOpen =
                        TunnelOpen::with_error("subdomain already in use".into());
                    write.send_delimited(data).await?;
                    return Ok(());
                }
                write.send_delimited(TunnelOpen::default()).await?;
                context.insert(
                    data.subdomain.clone().unwrap(),
                    Tunnel::with_event_conn(write).into(),
                );
                defer! {
                    context
                   .remove(&data.subdomain.clone().unwrap())
                }
                loop {
                    read.recv_delimited().await.context("client disconnected")?;
                }
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
                let mut set = JoinSet::new();

                set.spawn(async move {
                    listener.listen(context_clone).await.unwrap();
                    println!("exiting listener");
                });

                write.send_delimited(TunnelOpen::default()).await?;
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
                set.spawn(async move {
                    let _: Result<NewClient> =
                        read.recv_delimited().await.context("client disconnected");
                    ()
                });
                set.join_next().await;
                set.abort_all();
                Ok(())
            }
        }
    }
}

impl ControlServer {
    pub async fn new() -> Self {
        let addr = SocketAddr::from(([0, 0, 0, 0], SERVER_PORT));
        let listener = TcpListener::bind(addr).await.unwrap();
        info!(?addr, "server listening");
        Self { listener: listener }
    }
}

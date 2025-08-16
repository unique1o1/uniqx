use anyhow::{Context, Result};
use async_trait::async_trait;
use shared::delimited::{
    delimited_framed_read, delimited_framed_write, DelimitedReadExt, DelimitedWriteExt,
};
use shared::structs::NewClient;
use shared::utils::{set_tcp_keepalive, DeferCall};
use shared::Protocol;
use shared::{
    defer,
    structs::{TunnelOpen, TunnelRequest},
    utils::validate_subdomain,
    SERVER_PORT,
};
use socket2::SockRef;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tracing::info;

use crate::server::tcp_server::TcpServer;
use crate::tunnel::{SecureStream, Tunnel};
use crate::uniqx::ServerContext;
use tokio_rustls::TlsAcceptor;

use super::tcp_listener::{EventHandler, EventListener, TCPListener};

pub struct ControlServer {
    domain: String,
    listener: TcpListener,
    tls_acceptor: Option<TlsAcceptor>,
}
impl TCPListener for ControlServer {
    #[inline]
    fn listener(&self) -> &TcpListener {
        &self.listener
    }
}

#[async_trait]
impl EventHandler for ControlServer {
    async fn handle_conn(&self, stream: TcpStream, context: Arc<ServerContext>) -> Result<()> {
        let stream = if let Some(tls_acceptor) = &self.tls_acceptor {
            let stream = tls_acceptor.accept(stream).await?;
            SecureStream::Tls(stream)
        } else {
            SecureStream::Plain(stream)
        };

        self.handle_secure_conn(stream, context).await
    }
}
impl ControlServer {
    async fn handle_secure_conn(
        &self,
        stream: SecureStream,
        context: Arc<ServerContext>,
    ) -> Result<()> {
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
                    let data: TunnelOpen = TunnelOpen::with_error("subdomain already in use");
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
                    Tunnel::with_event_conn(write),
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
                if let Err(msg) = validate_subdomain(&data.subdomain) {
                    let data: TunnelOpen = TunnelOpen::with_error(&msg);
                    write.send_delimited(data).await?;
                }
                if context.contains_key(&data.tcp_port.unwrap().to_string()) {
                    write
                        .send_delimited(TunnelOpen::with_error("Port already in use"))
                        .await?;
                    return Ok(());
                }
                let listener = TcpServer::new(data.tcp_port.unwrap()).await;
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
                    Tunnel::with_event_conn(write),
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

    pub async fn new(domain: String, tls_acceptor: Option<TlsAcceptor>) -> Result<Self> {
        let addr = SocketAddr::from(([0, 0, 0, 0], SERVER_PORT));
        let listener = TcpListener::bind(addr).await?;

        SockRef::from(&listener)
            .set_tcp_keepalive(&set_tcp_keepalive())
            .unwrap();
        info!(?addr, "server listening");
        Ok(Self {
            listener,
            domain,
            tls_acceptor,
        })
    }
}

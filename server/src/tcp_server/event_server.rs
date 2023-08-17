use anyhow::Result;
use async_trait::async_trait;
use futures::executor::block_on;
use shared::utils::DeferCall;
use shared::{
    defer,
    frame::Delimited,
    structs::{Protocol, TunnelOpen, TunnelRequest},
    utils::validate_subdomain,
    SERVER_PORT,
};
use std::{sync::Arc, time::Duration};
use tokio::sync::RwLock;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::Mutex,
    time::sleep,
};
use tracing::{error, info, info_span, trace, warn, Instrument};

use crate::tcp_server::tcp_listener::TcpServer;
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
    async fn handle_conn(
        &self,
        stream: TcpStream,
        mut context: Arc<RwLock<ServerContext>>,
    ) -> Result<()> {
        stream.split()
        let t: Tunnel = Tunnel {
            event_conn: Some(Arc::new(Mutex::new(Delimited::new(stream)))),
            ..Default::default()
        };
        let stream = t.event_conn.clone().unwrap();
        let data: TunnelRequest = stream.lock().await.recv().await?;
        if let Err(msg) = validate_subdomain(&data.subdomain) {
            let data: TunnelOpen = TunnelOpen::with_error(msg);
            stream.lock().await.send(data).await?;
        }
        match data.protocol {
            Protocol::HTTP => {
                stream
                    .lock()
                    .await
                    .send(TunnelOpen {
                        error_message: None,
                    })
                    .await?;

                context
                    .write()
                    .await
                    .insert(data.subdomain.clone(), t.into());
                // defer! {
                //      block_on( context.write())
                //    .remove(&data.subdomain)
                // }

                loop {
                    stream.lock().await.recv().await?
                }
                // let tunnel = HttpTunnel::new(data.subdomain, data.port, stream).await?;
            }
            Protocol::TCP => {
                todo!()
            }
        }
        // Ok(())
    }
}

impl EventServer {
    pub async fn new() -> Self {
        let listener = TcpListener::bind(("0.0.0.0", SERVER_PORT)).await.unwrap();
        Self { listener: listener }
    }
}
impl TcpServer for EventServer {}

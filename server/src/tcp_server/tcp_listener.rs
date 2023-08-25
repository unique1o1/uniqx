use std::{io::ErrorKind, sync::Arc};

use async_trait::async_trait;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{Mutex, RwLock},
};
use tracing::{error, info, info_span, warn, Instrument};

use crate::uniq::ServerContext;
use anyhow::Result;
#[async_trait]
pub trait TCPListener {
    fn listener(&self) -> &TcpListener;
}
#[async_trait]
pub trait EventHandler {
    async fn handle_conn(&self, stream: TcpStream, context: Arc<ServerContext>) -> Result<()>;
}

#[async_trait]
pub trait EventListener: TCPListener + EventHandler {
    async fn listen(self, context: Arc<ServerContext>) -> Result<()>;
}
/// Blanket implementation for all types that implement `TCPListener` and `EventHandler`.
#[async_trait]

impl<T: TCPListener + EventHandler + Send + Sync + 'static> EventListener for T {
    async fn listen(self, context: Arc<ServerContext>) -> Result<()> {
        let this = Arc::new(self);
        loop {
            let this = this.clone();
            let (stream, addr) = if let Ok(x) = this.listener().accept().await {
                x
            } else {
                error!("failed to accept connection");
                continue;
            };
            let context = context.clone();
            tokio::spawn(
                async move {
                    if let Err(err) = this.handle_conn(stream, context).await {
                        error!(?err, "connection exited with error");
                    } else {
                        info!("connection exited");
                    }
                }
                .instrument(info_span!("control", ?addr)),
            );
        }
    }
}

/// A trait for types that can be used as a TCP server.
pub trait TcpServer: EventListener + EventHandler + TCPListener + Send + Sync {}
/// Blanket implementation for all types that implement `EventListener`.
impl<T: EventListener + Send + Sync + 'static> TcpServer for T {}

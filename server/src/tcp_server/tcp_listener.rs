use std::sync::Arc;

use async_trait::async_trait;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{Mutex, RwLock},
};
use tracing::{error, info, info_span, warn, Instrument};

use crate::uniq::ServerContext;
use anyhow::Result;
pub trait TCPListener {
    fn listener(&self) -> &TcpListener;
}
#[async_trait]
pub trait EventHandler {
    async fn handle_conn(
        &self,
        stream: TcpStream,
        context: Arc<RwLock<ServerContext>>,
    ) -> Result<()>;
}

#[async_trait]
pub trait EventListener: TCPListener + EventHandler {
    async fn listen(self, context: Arc<RwLock<ServerContext>>) -> Result<()>;
}
#[async_trait]
impl<T: TCPListener + EventHandler + Send + Sync + 'static> EventListener for T {
    async fn listen(self, context: Arc<RwLock<ServerContext>>) -> Result<()> {
        let self_clone = Arc::new(self);
        loop {
            let this = self_clone.clone();
            let (stream, addr) = {
                if let Ok(x) = this.listener().accept().await {
                    x
                } else {
                    error!("failed to accept connection");
                    continue;
                }
            };
            let context = context.clone();
            tokio::spawn(
                async move {
                    info!("incoming tunnel connection");
                    if let Err(err) = this.handle_conn(stream, context).await {
                        warn!(%err, "connection exited with error");
                    } else {
                        info!("connection exited");
                    }
                }
                .instrument(info_span!("control", ?addr)),
            );
        }
    }
}

pub trait TcpServer: EventListener + EventHandler + TCPListener + Send + Sync {}

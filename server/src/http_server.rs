use std::sync::Arc;
use tracing::{info, info_span, warn, Instrument};

use anyhow::{Error, Result};
use async_trait::async_trait;
use shared::SERVER_PORT;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

use crate::uniq::ServerContext;
#[async_trait]
pub trait Tunnel {}
pub struct HttpServer {
    listener: TcpListener,
}
impl HttpServer {
    pub async fn new() -> Self {
        let listener = TcpListener::bind(("0.0.0.0", 8001)).await.unwrap();
        Self {
            // listener: Arc::new(Mutex::new(listener)),
            listener: listener,
        }
    }
    async fn handle_conn(
        &self,
        stream: TcpStream,
        context: Arc<Mutex<ServerContext>>,
    ) -> Result<()> {
        todo!();
    }

    pub async fn listen(self, context: Arc<Mutex<ServerContext>>) -> Result<()> {
        let self_clone = Arc::new(self);
        loop {
            let this = self_clone.clone();
            let (stream, addr) = this.listener.accept().await?;
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

use anyhow::Result;
use shared::{
    frame::Delimited,
    structs::{Protocol, TunnelOpen, TunnelRequest},
    utils::validate_subdomain,
    SERVER_PORT,
};
use std::{future::Future, sync::Arc};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::Mutex,
};
use tracing::{info, info_span, trace, warn, Instrument};

use crate::{http_server::HttpServer, uniq::ServerContext};
#[derive(Default)]
pub struct Tunnel {
    pub event_conn: Option<Arc<Mutex<Delimited<TcpStream>>>>,
    pub private_http_conn: Option<Arc<Mutex<Delimited<TcpStream>>>>,
    pub public_http_conn: Option<Arc<Mutex<Delimited<TcpStream>>>>,
}
pub struct EventServer {
    listener: TcpListener,
}

impl EventServer {
    pub async fn new() -> Self {
        let listener = TcpListener::bind(("0.0.0.0", SERVER_PORT)).await.unwrap();
        Self {
            // listener: Arc::new(Mutex::new(listener)),
            listener: listener,
        }
    }
    async fn handle_conn(
        &self,
        mut stream: Delimited<TcpStream>,
        context: Arc<Mutex<ServerContext>>,
    ) -> Result<()> {
        let data: TunnelRequest = stream.recv().await?;
        if let Err(msg) = validate_subdomain(&data.subdomain) {
            let data = TunnelOpen::with_error(msg);
            trace!("sending` error response");
            stream.send(data).await?;
        }

        match data.protocol {
            Protocol::HTTP => {
                let t = Tunnel {
                    event_conn: Some(Arc::new(Mutex::new(stream))),
                    ..Default::default()
                };
                context.lock().await.http_tunnels.insert(data.subdomain, t);
                // let tunnel = HttpTunnel::new(data.subdomain, data.port, stream).await?;
            }
            Protocol::TCP => {
                todo!()
            }
        }

        Ok(())
    }
    pub async fn listen(self, context: Arc<Mutex<ServerContext>>) -> Result<()> {
        let self_clone = Arc::new(self);
        loop {
            let this = self_clone.clone();
            let (stream, addr) = this.listener.accept().await?;
            let context = context.clone();
            let stream: Delimited<TcpStream> = Delimited::new(stream);
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

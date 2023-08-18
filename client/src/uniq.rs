use std::sync::Arc;
use std::time::Duration;

use crate::Args;
use anyhow::Context;
use anyhow::Error;
use anyhow::Result;
use shared::connect_with_timeout;
use shared::frame::Delimited;
use shared::structs::NewClient;
use shared::structs::Protocol;
use shared::structs::TunnelOpen;
use shared::structs::TunnelRequest;
use shared::utils::proxy;
use shared::HTTP_EVENT_SERVER_PORT;
use shared::NETWORK_TIMEOUT;
use shared::SERVER_PORT;
use tokio::io;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::sync::OnceCell;
use tokio::time::timeout;
use tracing::error;
use tracing::info;
use tracing::trace;
pub(crate) struct UniqClient {
    local_port: u16,
    remote_host: String,
    local_host: String,
    protocol: Protocol,
    subdomain: String,
    conn: Option<Delimited<TcpStream>>,
}

impl UniqClient {
    pub async fn new(args: Args) -> Result<Self> {
        let stream = Delimited::new(connect_with_timeout(&args.remote_host, SERVER_PORT).await?);

        Ok(Self {
            local_port: args.local_port,
            remote_host: args.remote_host,
            local_host: args.local_host,
            conn: Some(stream),
            subdomain: args.subdomain,
            protocol: args.protocol,
        })
    }

    pub async fn handle_request(&self, data: NewClient) -> Result<()> {
        info!("connecting to local server and http event server");
        let localhost_conn = connect_with_timeout(&self.local_host, self.local_port).await?;
        let mut http_event_stream =
            connect_with_timeout(&self.remote_host, HTTP_EVENT_SERVER_PORT).await?;
        Delimited::new(&mut http_event_stream).send(data).await?;
        let (mut s1_read, mut s1_write) = io::split(localhost_conn);
        let (mut s2_read, mut s2_write) = io::split(http_event_stream);
        loop {
            tokio::select! {
                res = io::copy(&mut s1_read, &mut s2_write) => res,
                res = io::copy(&mut s2_read, &mut s1_write) => res,
            }?;
        }
        // Ok(())
    }

    pub async fn start(mut self) -> ! {
        let mut conn = self.conn.take().unwrap();
        let t = TunnelRequest {
            protocol: self.protocol.clone(),
            subdomain: self.subdomain.clone(),
        };
        if conn.send(t).await.is_err() {
            eprintln!("Unable to write to the remote server");
        }

        let data: TunnelOpen = conn.recv_timeout().await.unwrap();
        if data.error_message.is_some() {
            error!("Error: {}", data.error_message.unwrap());
            panic!("");
        }
        println!("Status: \t Online ");
        println!("Protocol: \t {:?}", self.protocol);
        println!(
            "Forwarded: \t {} -> {}",
            format!("http://{}.{}", self.subdomain, self.remote_host),
            format!("http://{}:{}", self.local_host, self.local_port),
        );
        let this: Arc<UniqClient> = Arc::new(self);
        loop {
            let data: NewClient = conn.recv().await.context("Connection timed out").unwrap();
            let this = this.clone();
            tokio::spawn(async move {
                this.handle_request(data).await.unwrap();
            });
        }
    }
}

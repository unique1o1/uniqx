use std::sync::Arc;
use std::time::Duration;

use crate::Args;
use anyhow::Context;
use anyhow::Result;
use shared::connect_with_timeout;
use shared::frame::Delimited;
use shared::structs::NewClient;
use shared::structs::Protocol;
use shared::structs::TunnelOpen;
use shared::structs::TunnelRequest;
use shared::HTTP_EVENT_SERVER_PORT;
use shared::NETWORK_TIMEOUT;
use shared::SERVER_PORT;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::timeout;
use tracing::error;
use tracing::info;
use tracing::trace;

pub(crate) struct UniqClient {
    local_port: u16,
    remote_host: String,
    local_host: String,
    remote_port: u16,
    protocol: Protocol,
    subdomain: String,
    conn: Delimited<TcpStream>,
}

impl UniqClient {
    pub async fn new(args: Args) -> Result<Self> {
        let stream = Delimited::new(connect_with_timeout(&args.remote_host, SERVER_PORT).await?);

        Ok(Self {
            local_port: args.local_port,
            remote_port: SERVER_PORT,
            remote_host: args.remote_host,
            local_host: args.local_host,
            conn: stream,
            subdomain: args.subdomain,
            protocol: args.protocol,
        })
    }

    pub async fn handle_request(&self, data: NewClient) -> Result<()> {
        info!("New client: {}", data.identifier);
        let http_event_conn =
            connect_with_timeout(&self.remote_host, HTTP_EVENT_SERVER_PORT).await?;
        let localhost_conn =
            connect_with_timeout(&self.remote_host, HTTP_EVENT_SERVER_PORT).await?;
        println!("New client: {}", data.identifier);
        Ok(())
    }

    pub async fn start(mut self) -> Result<()> {
        let t = TunnelRequest {
            protocol: self.protocol.clone(),
            subdomain: self.subdomain.clone(),
        };
        if self.conn.send(t).await.is_err() {
            eprintln!("Unable to write to the remote server");
        }

        let data: TunnelOpen = self.conn.recv_timeout().await?;
        if data.error_message.is_some() {
            error!("Error: {}", data.error_message.unwrap());
            return Ok(());
        }
        println!("Status: \t Online ");
        println!("Protocol: \t {:?}", self.protocol);
        println!(
            "Forwarded: \t {} -> {}",
            format!("http://{}.{}", self.subdomain, self.remote_host),
            format!("http://{}:{}", self.local_host, self.local_port),
        );
        let mut this = Arc::new(Mutex::new(self));
        loop {
            let data: NewClient = this
                .lock()
                .await
                .conn
                .recv_timeout()
                .await
                .context("Connection timed out")?;
            let this = this.clone();
            tokio::spawn(async move { this.lock().await.handle_request(data).await });
        }
    }
}

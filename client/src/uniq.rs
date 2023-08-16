use anyhow::Context;
use anyhow::Result;
use shared::connect_with_timeout;
use shared::frame::Delimited;
use shared::structs::Protocol;
use shared::structs::TunnelOpen;
use shared::structs::TunnelRequest;
use shared::NETWORK_TIMEOUT;
use shared::SERVER_PORT;
use tokio::net::TcpStream;
use tokio::time::timeout;
use tracing::trace;

use crate::Args;

pub(crate) struct UniqClient {
    local_port: u16,
    remote_addr: String,
    local_addr: String,
    remote_port: u16,
    protocol: Protocol,
    subdomain: String,
    conn: Delimited<TcpStream>,
}

impl UniqClient {
    pub async fn new(args: Args) -> Result<Self> {
        let local_addr = format!("{}:{}", args.local_host, args.local_port);
        let remote_addr = format!("{}:{}", args.remote_host, SERVER_PORT);
        let stream = Delimited::new(connect_with_timeout(&args.remote_host, SERVER_PORT).await?);

        Ok(Self {
            local_port: args.local_port,
            remote_port: SERVER_PORT,
            remote_addr,
            local_addr,
            conn: stream,
            subdomain: args.subdomain,
            protocol: args.protocol,
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        let t = TunnelRequest {
            protocol: self.protocol.clone(),
            subdomain: self.subdomain.clone(),
        };
        if self.conn.send(t).await.is_err() {
            eprintln!("Unable to write to the remote server");
        }

        let data: TunnelOpen = self.conn.recv_timeout().await?;
        println!("Received data: {:?}", data);
        println!("Status: \t Online ");
        println!("Protocol: \t {:?}", self.protocol);
        println!(
            "Forwarded: \t {} -> {}",
            data.hostname.unwrap(),
            self.local_addr
        );
        return Ok(());
    }
}

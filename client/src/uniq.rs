use std::process::exit;
use std::sync::Arc;

use crate::Args;
use anyhow::Context;
use anyhow::Result;
use shared::connect_with_timeout;
use shared::delimited::delimited_framed;
use shared::delimited::DelimitedReadExt;
use shared::delimited::DelimitedStream;
use shared::delimited::DelimitedWriteExt;
use shared::structs::NewClient;
use shared::structs::Protocol;
use shared::structs::TunnelOpen;
use shared::structs::TunnelRequest;
use shared::utils::bind;
use shared::utils::proxy;
use shared::HTTP_EVENT_SERVER_PORT;
use shared::SERVER_PORT;
use tracing::error;
use tracing::info;
use tracing::trace;
pub(crate) struct UniqClient {
    local_port: u16,
    remote_host: String,

    local_host: String,
    protocol: Protocol,
    subdomain: Option<String>,
    port: Option<u16>,
    conn: Option<DelimitedStream>,
}

impl UniqClient {
    pub async fn new(args: Args) -> Result<Self> {
        let stream = delimited_framed(connect_with_timeout(&args.remote_host, SERVER_PORT).await?);

        Ok(Self {
            local_port: args.local_port,
            remote_host: args.remote_host,
            port: args.port,
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
        delimited_framed(&mut http_event_stream)
            .send_delimited(data)
            .await?;
        // let (s1_read, s1_write) = io::split(localhost_conn);
        // let (s2_read, s2_write) = io::split(http_event_stream);
        // tokio::spawn(async move { bind(s1_read, s2_write).await.context("cant read from s1") });
        // bind(s2_read, s1_write).await.context("cant read from s2")?;
        proxy(localhost_conn, http_event_stream).await?;
        // proxy(http_event_stream, localhost_conn);
        Ok(())
    }

    pub async fn start(mut self) -> ! {
        let mut conn = self.conn.take().unwrap();
        let t = TunnelRequest {
            tcp_port: self.port,
            protocol: self.protocol.clone(),
            subdomain: self.subdomain.clone(),
        };
        if conn.send_delimited(t).await.is_err() {
            eprintln!("Unable to write to the remote server");
        }
        let data: TunnelOpen = conn.recv_timeout_delimited().await.unwrap();
        if data.error_message.is_some() {
            error!("Error: {}", data.error_message.unwrap());
            exit(1)
        }

        println!("Status: \t Online ");
        println!("Protocol: \t {:?}", self.protocol);

        println!(
            "Forwarded: \t {} -> {}",
            format!(
                "https://{}.{}:{}",
                self.subdomain.as_ref().unwrap(),
                self.remote_host,
                self.port.unwrap_or(443),
            ),
            format!("http://{}:{}", self.local_host, self.local_port),
        );
        let this: Arc<UniqClient> = Arc::new(self);
        loop {
            println!("Waiting for new connection");
            let data: NewClient = conn.recv_delimited().await.unwrap();
            let this = this.clone();
            tokio::spawn(async move {
                this.handle_request(data).await.unwrap();
            });
        }
        // match self.protocol {
        //     Protocol::HTTP => {}
        //     Protocol::TCP => loop {},
        // }
    }
}

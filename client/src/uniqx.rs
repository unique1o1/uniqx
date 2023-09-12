use anyhow::Result;
use shared::connect_with_timeout;
use shared::delimited::delimited_framed;
use shared::delimited::DelimitedReadExt;
use shared::delimited::DelimitedStream;
use shared::delimited::DelimitedWriteExt;
use shared::structs::NewClient;
use shared::structs::TunnelOpen;
use shared::structs::TunnelRequest;
use shared::utils::proxy;
use shared::utils::set_tcp_keepalive;
use shared::Protocol;
use shared::EVENT_SERVER_PORT;
use shared::SERVER_PORT;
use socket2::SockRef;
use std::process::exit;
use std::sync::Arc;
use tokio::io::{self};
use tracing::error;
use tracing::info;
use tracing::info_span;
use tracing::Instrument;

use crate::console;
use crate::console::handler::ConsoleHandler;
use crate::util::bind_with_console;

pub struct UniqxClient {
    local_port: u16,
    remote_host: String,
    local_host: String,
    protocol: Protocol,
    subdomain: String,
    port: Option<u16>,
    console: bool,
    conn: Option<DelimitedStream>,
    console_handler: Option<ConsoleHandler>,
}

impl UniqxClient {
    pub async fn new(
        protocol: Protocol,
        local_port: u16,
        port: Option<u16>,
        remote_host: String,
        subdomain: String,
        local_host: String,
        console: bool,
    ) -> Result<Self> {
        let conn = connect_with_timeout(&remote_host, SERVER_PORT).await?;

        SockRef::from(&conn)
            .set_tcp_keepalive(&set_tcp_keepalive())
            .unwrap();
        let stream = delimited_framed(conn);

        Ok(Self {
            local_port,
            remote_host,
            port,
            local_host,
            subdomain,
            protocol,
            console,
            conn: Some(stream),
            console_handler: None,
        })
    }

    pub async fn handle_request(&self, data: NewClient) -> Result<()> {
        let localhost_conn = connect_with_timeout(&self.local_host, self.local_port).await?;
        let mut http_event_stream =
            connect_with_timeout(&self.remote_host, EVENT_SERVER_PORT).await?;
        delimited_framed(&mut http_event_stream)
            .send_delimited(data)
            .await?;

        if self.protocol == Protocol::HTTP && self.console {
            let (s1_read, s1_write) = io::split(localhost_conn);
            let (s2_read, s2_write) = io::split(http_event_stream);
            let (req_tx, res_tx) = self.console_handler.clone().unwrap().init_transmitter();
            tokio::select! {
                res= bind_with_console(s1_read, s2_write, res_tx).instrument(info_span!("Binder", "localhost reader")) => { info!("local connection discounted");res},
                res= bind_with_console(s2_read, s1_write, req_tx).instrument(info_span!("Binder", "http event reader")) =>  {info!("event connection discounted"); res}
            }?
        } else {
            proxy(localhost_conn, http_event_stream).await?;
        }

        Ok(())
    }

    pub async fn start(mut self) -> Result<()> {
        let mut conn = self.conn.take().unwrap();
        let t = TunnelRequest {
            tcp_port: self.port,
            protocol: self.protocol.clone(),
            subdomain: self.subdomain.clone(),
        };
        if conn.send_delimited(t).await.is_err() {
            error!("Unable to write to the remote server");
        }
        let data: TunnelOpen = conn.recv_timeout_delimited().await.unwrap();
        if data.error_message.is_some() {
            error!("Error: {}", data.error_message.unwrap());
            exit(1)
        }

        println!("Status: \t Online ");
        println!("Protocol: \t {:?}", self.protocol);

        println!(
            "Forwarded: \t {}:{} -> {}:{}",
            data.access_point,
            self.port.unwrap_or(443),
            self.local_host,
            self.local_port
        );
        if self.console {
            self.console_handler = Some(console::server::start().await);

            println!(
                "Console: \t http://{}:{}",
                self.local_host,
                self.console_handler.as_ref().unwrap().port
            );
        }
        let this: Arc<UniqxClient> = Arc::new(self);
        loop {
            let data: NewClient = conn.recv_delimited().await?;
            let this = this.clone();
            let identifier = data.public_conn_identifier.clone();
            tokio::spawn(
                async move { this.handle_request(data).await }
                    .instrument(info_span!("control", ?identifier)),
            );
        }
    }
}

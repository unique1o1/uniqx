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
use shared::Protocol;
use shared::EVENT_SERVER_PORT;
use shared::SERVER_PORT;
use std::process::exit;
use std::sync::Arc;
use tokio::io::{self};
use tracing::error;
use tracing::info;
use tracing::info_span;
use tracing::Instrument;

use anyhow::Context;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio_rustls::{
    rustls::{self, ClientConfig, OwnedTrustAnchor},
    TlsConnector,
};

use crate::console;
use crate::console::handler::ConsoleHandler;
use crate::util::bind_with_console;
use std::pin::Pin;
use std::task::{Context as TaskContext, Poll};
use tokio_rustls::client::TlsStream;

enum SecureStream {
    Plain(TcpStream),
    Tls(TlsStream<TcpStream>),
}

impl AsyncRead for SecureStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut TaskContext<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        match &mut *self {
            SecureStream::Plain(stream) => Pin::new(stream).poll_read(cx, buf),
            SecureStream::Tls(stream) => Pin::new(stream).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for SecureStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut TaskContext<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        match &mut *self {
            SecureStream::Plain(stream) => Pin::new(stream).poll_write(cx, buf),
            SecureStream::Tls(stream) => Pin::new(stream).poll_write(cx, buf),
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<Result<(), io::Error>> {
        match &mut *self {
            SecureStream::Plain(stream) => Pin::new(stream).poll_flush(cx),
            SecureStream::Tls(stream) => Pin::new(stream).poll_flush(cx),
        }
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<Result<(), io::Error>> {
        match &mut *self {
            SecureStream::Plain(stream) => Pin::new(stream).poll_shutdown(cx),
            SecureStream::Tls(stream) => Pin::new(stream).poll_shutdown(cx),
        }
    }
}

pub struct UniqxClient {
    local_port: u16,
    remote_host: String,
    local_host: String,
    protocol: Protocol,
    subdomain: String,
    port: Option<u16>,
    console: bool,
    conn: Option<DelimitedStream<SecureStream>>,
    console_handler: Option<ConsoleHandler>,
    tls: bool,
    insecure: bool,
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
        tls: bool,
        insecure: bool,
    ) -> Result<Self> {
        let stream = if tls {
            if insecure {
                // custom verifier that trusts any cert
                struct DangerousServerCertVerifier;
                impl rustls::client::ServerCertVerifier for DangerousServerCertVerifier {
                    fn verify_server_cert(
                        &self,
                        _end_entity: &rustls::Certificate,
                        _intermediates: &[rustls::Certificate],
                        _server_name: &rustls::ServerName,
                        _scts: &mut dyn Iterator<Item = &[u8]>,
                        _ocsp_response: &[u8],
                        _now: std::time::SystemTime,
                    ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
                        Ok(rustls::client::ServerCertVerified::assertion())
                    }
                }
                let config = ClientConfig::builder()
                    .with_safe_defaults()
                    .with_custom_certificate_verifier(Arc::new(DangerousServerCertVerifier {}))
                    .with_no_client_auth();
                let connector = TlsConnector::from(Arc::new(config));
                let stream = connect_with_timeout(&remote_host, SERVER_PORT).await?;
                let domain = rustls::ServerName::try_from(remote_host.as_str())
                    .with_context(|| format!("Invalid DNS name: {}", remote_host))?;
                let stream = connector.connect(domain, stream).await?;
                SecureStream::Tls(stream)
            } else {
                let mut root_cert_store = rustls::RootCertStore::empty();
                root_cert_store.add_trust_anchors(
                    webpki_roots::TLS_SERVER_ROOTS.iter().map(|ta| {
                        OwnedTrustAnchor::from_subject_spki_name_constraints(
                            ta.subject,
                            ta.spki,
                            ta.name_constraints,
                        )
                    }),
                );
                let config = ClientConfig::builder()
                    .with_safe_defaults()
                    .with_root_certificates(root_cert_store)
                    .with_no_client_auth();
                let connector = TlsConnector::from(Arc::new(config));
                let stream = connect_with_timeout(&remote_host, SERVER_PORT).await?;
                let domain = rustls::ServerName::try_from(remote_host.as_str())
                    .with_context(|| format!("Invalid DNS name: {}", remote_host))?;
                let stream = connector.connect(domain, stream).await?;
                SecureStream::Tls(stream)
            }
        } else {
            let stream = connect_with_timeout(&remote_host, SERVER_PORT).await?;
            SecureStream::Plain(stream)
        };

        // SockRef::from(&conn)
        //     .set_tcp_keepalive(&set_tcp_keepalive())
        //     .unwrap();
        let stream = delimited_framed(stream);

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
            tls,
            insecure,
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

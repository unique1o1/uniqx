use anyhow::{ensure, Context, Ok, Result};
use dashmap::DashMap;
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use tokio_rustls::{
    rustls::{self, Certificate, PrivateKey},
    TlsAcceptor,
};
use tracing::info;

use crate::{
    server::{
        control_server::ControlServer, event_server::EventServer, http_server::HttpServer,
        tcp_listener::TcpServer,
    },
    tunnel::Tunnel,
};
pub(crate) type ServerContext = DashMap<String, Tunnel>;

pub struct UniqxServer {
    domain: String,
    http_port: u16,
    server_context: Arc<ServerContext>,
    tls_acceptor: Option<TlsAcceptor>,
}
impl UniqxServer {
    pub async fn new(
        domain: String,
        http_port: u16,
        cert_path: Option<String>,
        key_path: Option<String>,
    ) -> Result<UniqxServer> {
        let tls_acceptor = if let (Some(cert_path), Some(key_path)) = (cert_path, key_path) {
            let certs = load_certs(&cert_path)?;
            let key = load_private_key(&key_path)?;

            let config = rustls::ServerConfig::builder()
                .with_safe_defaults()
                .with_no_client_auth()
                .with_single_cert(certs, key)
                .with_context(|| format!("Failed to create TLS config with cert: {} and key: {}", cert_path, key_path))?;

            Some(TlsAcceptor::from(Arc::new(config)))
        } else {
            None
        };

        Ok(UniqxServer {
            domain,
            server_context: Arc::new(ServerContext::default()),
            http_port,
            tls_acceptor,
        })
    }

    // Start the server, listening for new connections.
    fn listen<S: TcpServer + 'static>(&self, event_server: S) {
        let context = self.server_context.clone();
        tokio::spawn(async move {
            event_server.listen(context).await.unwrap();
            info!("exiting listener");
        });
    }

    pub async fn start(self) -> Result<()> {
        self.listen(ControlServer::new(self.domain.clone(), self.tls_acceptor.clone()).await?);
        self.listen(HttpServer::new(self.http_port).await?);
        self.listen(EventServer::new().await?);

        Ok(())
    }
}

fn load_certs(path: &str) -> Result<Vec<Certificate>> {
    let mut cert_file = BufReader::new(File::open(path).with_context(|| format!("Failed to open cert file: {}", path))?);
    let certs = certs(&mut cert_file)
        .with_context(|| format!("Failed to parse cert file: {}", path))?
        .into_iter()
        .map(Certificate)
        .collect();
    Ok(certs)
}

fn load_private_key(path: &str) -> Result<PrivateKey> {
    let mut key_file = BufReader::new(File::open(path).with_context(|| format!("Failed to open key file: {}", path))?);
    let mut keys = pkcs8_private_keys(&mut key_file)
        .with_context(|| format!("Failed to parse key file: {}", path))?;
    ensure!(keys.len() == 1, "Expected a single private key");
    Ok(PrivateKey(keys.remove(0)))
}

//! Shared data structures, utilities, and protocol definitions.
use std::time::Duration;
/// TCP port used for control connections with the server.
pub const CONTROL_PORT: u16 = 7835;

/// Maximum byte length for a JSON frame in the stream.
pub const MAX_FRAME_LENGTH: usize = 256;

/// Timeout for network connections and initial protocol messages.
pub const NETWORK_TIMEOUT: Duration = Duration::from_secs(10);
/// Port used for control server's TCP socket.
pub const SERVER_PORT: u16 = 9876;
//  Port used for event server's TCP socket.
pub const EVENT_SERVER_PORT: u16 = 9875;
// specifies the time (in seconds) that the connection must remain idle before TCP starts sending keepalive probes.
pub const TCP_KEEPIDLE: u64 = 30;
// specifies the time (in seconds) between individual keepalive probes.
pub const TCP_KEEPINTVL: u64 = 15;
// specifies the maximum number of keepalive probes TCP should send before dropping the connection.
pub const TCP_KEEPCNT: u32 = 6;

use anyhow::{Context, Result};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use tokio::{net::TcpStream, time::timeout};

#[derive(Serialize, Deserialize, Debug, Clone, ValueEnum)]
pub enum Protocol {
    HTTP,
    TCP,
}

pub mod delimited;
pub mod structs;
pub mod utils;

pub async fn connect_with_timeout(to: &str, port: u16) -> Result<TcpStream> {
    match timeout(NETWORK_TIMEOUT, TcpStream::connect((to, port))).await {
        Ok(res) => res,
        Err(err) => Err(err.into()),
    }
    .with_context(|| format!("could not connect to {to}"))
}

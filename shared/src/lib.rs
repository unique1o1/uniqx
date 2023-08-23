//! Shared data structures, utilities, and protocol definitions.
/// TCP port used for control connections with the server.
pub const CONTROL_PORT: u16 = 7835;

/// Maximum byte length for a JSON frame in the stream.
pub const MAX_FRAME_LENGTH: usize = 256;

/// Timeout for network connections and initial protocol messages.
pub const NETWORK_TIMEOUT: Duration = Duration::from_secs(10);
/// Port used for the server's TCP socket.
pub const SERVER_PORT: u16 = 9876;
pub const HTTP_EVENT_SERVER_PORT: u16 = 9875;
use std::time::Duration;

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
pub mod events;
pub mod structs;
pub mod utils;

pub async fn connect_with_timeout(to: &str, port: u16) -> Result<TcpStream> {
    match timeout(NETWORK_TIMEOUT, TcpStream::connect((to, port))).await {
        Ok(res) => res,
        Err(err) => Err(err.into()),
    }
    .with_context(|| format!("could not connect to {to}"))
}

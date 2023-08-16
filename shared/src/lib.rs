use std::time::Duration;

use anyhow::{Context, Result};
use tokio::{io::AsyncReadExt, net::TcpStream, time::timeout};

pub mod events;
pub mod frame;
pub mod structs;
pub mod utils;
/// TCP port used for control connections with the server.
pub const CONTROL_PORT: u16 = 7835;

/// Maximum byte length for a JSON frame in the stream.
pub const MAX_FRAME_LENGTH: usize = 256;

/// Timeout for network connections and initial protocol messages.
pub const NETWORK_TIMEOUT: Duration = Duration::from_secs(10);
/// Port used for the server's TCP socket.
pub const SERVER_PORT: u16 = 9876;

pub async fn connect_with_timeout(to: &str, port: u16) -> Result<TcpStream> {
    match timeout(NETWORK_TIMEOUT, TcpStream::connect((to, port))).await {
        Ok(res) => res,
        Err(err) => Err(err.into()),
    }
    .with_context(|| format!("could not connect to {to}"))
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}

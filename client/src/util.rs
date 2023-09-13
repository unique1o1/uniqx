use anyhow::Result;

use std::fmt::Debug;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf},
    net::TcpStream,
};
use tracing::info;

use crate::console::handler::Transmitter;

pub async fn bind(mut src: ReadHalf<TcpStream>, mut dst: WriteHalf<TcpStream>) -> Result<()> {
    let mut buf = [0; 4096];
    loop {
        let n = src.read(&mut buf).await?;
        if n == 0 {
            info!("connection disconnected");
            return Ok(());
        }
        dst.write_all(&buf[..n]).await?;
    }
}
pub async fn bind_with_console<T: Transmitter + Debug>(
    mut src: ReadHalf<TcpStream>,
    mut dst: WriteHalf<TcpStream>,
    sender: T,
) -> Result<()> {
    let mut was_segmented = false;
    let mut console_data: Vec<u8> = vec![];
    let mut req_count = 0;
    loop {
        let mut buf = [0u8; 4096];

        let n: usize = src.read(&mut buf).await?;
        if n == 0 {
            return Ok(());
        }
        dst.write_all(&buf[..n]).await?;
        if !was_segmented {
            req_count += 1; // only increment req_count if the previous request was not segmented
            console_data.clear();
        }
        console_data.extend(&buf[..n]);
        was_segmented = sender
            .send(console_data.clone(), req_count)
            .await
            .unwrap_or(false);
    }
}

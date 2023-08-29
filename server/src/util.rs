use anyhow::Result;
use tokio::io::{self, AsyncRead, AsyncWrite, AsyncWriteExt};
use tracing::info;

pub async fn write_response(
    mut conn: impl AsyncWriteExt + Unpin,
    status_code: u16,
    status: &str,
    message: &str,
) -> Result<()> {
    let response = format!(
        "HTTP/1.1 {} {}\r\nContent-Length: {}\r\n\r\n{}",
        status_code,
        status,
        message.len(),
        message
    );
    conn.write_all(response.as_bytes()).await?;
    conn.flush().await?;
    Ok(())
}

/// Copy data mutually between two read/write streams.
pub async fn proxy<S1, S2>(stream1: S1, stream2: S2) -> io::Result<()>
where
    S1: AsyncRead + AsyncWrite + Unpin,
    S2: AsyncRead + AsyncWrite + Unpin,
{
    let (mut s1_read, mut s1_write) = io::split(stream1);
    let (mut s2_read, mut s2_write) = io::split(stream2);
    tokio::select! {
        res = io::copy(&mut s1_read, &mut s2_write) => { info!("local connection discounted"); res },
        res = io::copy(&mut s2_read, &mut s1_write) =>  {info!("event connection discounted");  res }
    }?;
    Ok(())
}

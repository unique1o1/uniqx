use anyhow::Result;
use tokio::io::AsyncWriteExt;

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

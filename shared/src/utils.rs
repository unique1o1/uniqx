use anyhow::Result;
use regex::Regex;
use tokio::io::{self, AsyncRead, AsyncWrite, AsyncWriteExt};
static BLOCK_LIST: &[&str] = &["www", "uniq"];
pub fn validate_subdomain(subdomain: &str) -> Result<(), String> {
    let regex = Regex::new(r"^[a-z\d](?:[a-z\d]|-[a-z\d]){0,38}$").unwrap();
    if subdomain.len() > 38 || subdomain.len() < 3 {
        return Err(String::from("subdomain length must be between 3 and 42"));
    }
    if BLOCK_LIST.contains(&subdomain) {
        return Err(String::from("subdomain is in deny list"));
    }
    if !regex.is_match(subdomain) {
        return Err(String::from("subdomain must be lowercase & alphanumeric"));
    }
    Ok(())
}

pub struct DeferCall<F: FnMut()> {
    pub c: F,
}
impl<F: FnMut()> Drop for DeferCall<F> {
    fn drop(&mut self) {
        (self.c)();
    }
}

#[macro_export]
macro_rules! defer {
    ($e:expr) => {
        let _scope_call = DeferCall {
            c: || -> () {
                $e;
            },
        };
    };
}
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
        res = io::copy(&mut s1_read, &mut s2_write) => res,
        res = io::copy(&mut s2_read, &mut s1_write) => res,
    }?;
    Ok(())
}

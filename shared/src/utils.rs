use std::time::Duration;

use crate::{TCP_KEEPCNT, TCP_KEEPIDLE, TCP_KEEPINTVL};
use anyhow::Result;
use regex::Regex;
use socket2::TcpKeepalive;
use tokio::io::{self, AsyncRead, AsyncWrite};
use tracing::info;
// use url::Url;
static BLOCK_LIST: &[&str] = &["www", "uniqx"];

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

#[inline]
#[cfg(target_os = "windows")]
pub fn set_tcp_keepalive() -> TcpKeepalive {
    TcpKeepalive::new()
        .with_time(Duration::from_secs(TCP_KEEPIDLE))
        .with_interval(Duration::from_secs(TCP_KEEPINTVL))
}
#[inline]
#[cfg(not(target_os = "windows"))]
pub fn set_tcp_keepalive() -> TcpKeepalive {
    TcpKeepalive::new()
        .with_time(Duration::from_secs(TCP_KEEPIDLE))
        .with_interval(Duration::from_secs(TCP_KEEPINTVL))
        .with_retries(TCP_KEEPCNT)
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

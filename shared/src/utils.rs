use std::time::Duration;

use crate::{TCP_KEEPCNT, TCP_KEEPIDLE, TCP_KEEPINTVL};
use anyhow::Result;
use regex::Regex;
use socket2::TcpKeepalive;
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

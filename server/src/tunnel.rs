use dashmap::DashMap;
use shared::delimited::DelimitedWriteStream;
use std::fmt::Debug;
use std::io;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context as TaskContext, Poll};
use tokio::io::{AsyncRead, AsyncWrite, WriteHalf};
use tokio::{net::TcpStream, sync::Mutex};
use tokio_rustls::server::TlsStream;

pub enum SecureStream {
    Plain(TcpStream),
    Tls(TlsStream<TcpStream>),
}

impl Debug for SecureStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Plain(_) => f.debug_tuple("Plain").finish(),
            Self::Tls(_) => f.debug_tuple("Tls").finish(),
        }
    }
}

impl AsyncRead for SecureStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut TaskContext<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        match &mut *self {
            SecureStream::Plain(stream) => Pin::new(stream).poll_read(cx, buf),
            SecureStream::Tls(stream) => Pin::new(stream).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for SecureStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut TaskContext<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        match &mut *self {
            SecureStream::Plain(stream) => Pin::new(stream).poll_write(cx, buf),
            SecureStream::Tls(stream) => Pin::new(stream).poll_write(cx, buf),
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<Result<(), io::Error>> {
        match &mut *self {
            SecureStream::Plain(stream) => Pin::new(stream).poll_flush(cx),
            SecureStream::Tls(stream) => Pin::new(stream).poll_flush(cx),
        }
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<Result<(), io::Error>> {
        match &mut *self {
            SecureStream::Plain(stream) => Pin::new(stream).poll_shutdown(cx),
            SecureStream::Tls(stream) => Pin::new(stream).poll_shutdown(cx),
        }
    }
}

#[derive(Debug)]
pub struct Tunnel {
    pub control_conn: Arc<Mutex<DelimitedWriteStream<WriteHalf<SecureStream>>>>,
    pub public_conn: Arc<DashMap<String, TcpStream>>,
}
impl Tunnel {
    pub(crate) fn with_event_conn(
        write: DelimitedWriteStream<WriteHalf<SecureStream>>,
    ) -> Self {
        Tunnel {
            control_conn: Arc::new(Mutex::new(write)),
            public_conn: Default::default(),
        }
    }
}

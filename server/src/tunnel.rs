use dashmap::DashMap;
use shared::delimited::{DelimitedReadStream, DelimitedWriteStream};
use std::sync::Arc;
use tokio::{io::WriteHalf, net::TcpStream, sync::Mutex};
use tokio_util::codec::{AnyDelimiterCodec, FramedWrite};

#[derive(Debug)]
pub struct Tunnel {
    pub event_conn: Arc<Mutex<DelimitedWriteStream>>,
    pub public_conn: Arc<DashMap<String, TcpStream>>,
    // pub initial_http_buffer: Arc<DashMap<String, Vec<u8>>>,
}
impl Tunnel {
    pub(crate) fn with_event_conn(write: DelimitedWriteStream) -> Self {
        Tunnel {
            event_conn: Arc::new(Mutex::new(write)),
            public_conn: Default::default(),
            // initial_http_buffer: Default::default(),
        }
    }
}
impl From<Tunnel> for Arc<Mutex<Tunnel>> {
    fn from(tunnel: Tunnel) -> Arc<Mutex<Tunnel>> {
        Arc::new(Mutex::new(tunnel))
    }
}

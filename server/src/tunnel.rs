use dashmap::DashMap;
use shared::delimited::DelimitedWriteStream;
use std::sync::Arc;
use tokio::{net::TcpStream, sync::Mutex};

#[derive(Debug)]
pub struct Tunnel {
    pub control_conn: Arc<Mutex<DelimitedWriteStream>>,
    pub public_conn: Arc<DashMap<String, TcpStream>>,
}
impl Tunnel {
    pub(crate) fn with_event_conn(write: DelimitedWriteStream) -> Self {
        Tunnel {
            control_conn: Arc::new(Mutex::new(write)),
            public_conn: Default::default(),
        }
    }
}

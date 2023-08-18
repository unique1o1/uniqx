use dashmap::DashMap;
use shared::frame::DelimitedWrite;
use std::{collections::HashMap, sync::Arc};
use tokio::{
    io::{ReadHalf, WriteHalf},
    net::TcpStream,
    sync::Mutex,
};

#[derive(Debug)]
pub struct Tunnel {
    pub event_conn: Arc<Mutex<DelimitedWrite>>,
    pub public_http_conn: Arc<DashMap<String, TcpStream>>,
    pub initialBuffer: Arc<DashMap<String, Vec<u8>>>,
}
impl Tunnel {
    pub(crate) fn with_event_conn(write: DelimitedWrite) -> Self {
        Tunnel {
            event_conn: Arc::new(Mutex::new(write)),
            public_http_conn: Default::default(),
            initialBuffer: Default::default(),
        }
    }
}
impl From<Tunnel> for Arc<Mutex<Tunnel>> {
    fn from(tunnel: Tunnel) -> Arc<Mutex<Tunnel>> {
        Arc::new(Mutex::new(tunnel))
    }
}

use std::{collections::HashMap, sync::Arc};

use shared::frame::Delimited;
use tokio::{
    net::TcpStream,
    sync::{Mutex, RwLock},
};

#[derive(Default, Debug)]
pub struct Tunnel {
    pub event_conn: Option<Arc<Mutex<Delimited<TcpStream>>>>,
    pub private_http_conn: Option<Arc<Mutex<TcpStream>>>,
    pub public_http_conn: Arc<Mutex<HashMap<String, TcpStream>>>,
}

impl From<Tunnel> for Arc<Mutex<Tunnel>> {
    fn from(tunnel: Tunnel) -> Arc<Mutex<Tunnel>> {
        Arc::new(Mutex::new(tunnel))
    }
}

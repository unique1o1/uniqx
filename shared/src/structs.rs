use serde::{Deserialize, Serialize};

use crate::Protocol;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TunnelRequest {
    pub protocol: Protocol,
    pub subdomain: Option<String>,
    pub tcp_port: Option<u16>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct TunnelOpen {
    pub error_message: Option<String>,
}
impl TunnelOpen {
    pub fn with_error(msg: &str) -> Self {
        let mut data = Self::default();
        data.error_message = Some(msg.to_owned());
        data
    }
}
#[derive(Serialize, Deserialize)]
pub struct NewClient {
    // pub client_ip: IpAddr,
    pub initial_buffer: Option<Vec<u8>>,
    pub public_conn_identifier: String,
    pub control_server_identifier: Option<String>,
}

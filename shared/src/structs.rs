use async_trait::async_trait;
use clap::ValueEnum;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::net::IpAddr;
use std::pin::Pin;
use std::rc::Rc;
use std::string;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use tokio_serde::{formats::Bincode, Framed};
#[async_trait]
pub trait Event {
    fn encode(&self) -> Vec<u8>;
    fn write<D: AsyncWriteExt + Unpin>(&self, writer: Rc<D>) -> std::io::Result<()>;
    async fn read<D: AsyncReadExt + Unpin>(reader: Rc<D>) -> Self;
}
#[derive(Serialize, Deserialize, Debug, Clone, ValueEnum)]
pub enum Protocol {
    HTTP,
    TCP,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TunnelRequest {
    pub protocol: Protocol,
    pub subdomain: String,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct TunnelOpen {
    pub error_message: Option<String>,
}
impl TunnelOpen {
    pub fn with_error(msg: String) -> Self {
        let mut data = Self::default();
        data.error_message = Some(msg);
        data
    }
}
#[derive(Serialize, Deserialize)]
pub struct NewClient {
    // pub client_ip: IpAddr,
    // pub client_port: u16,
    pub identifier: String,
}

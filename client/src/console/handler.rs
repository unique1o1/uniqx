use actix_web::{body::to_bytes, web::Bytes};
use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::broadcast::Sender;
use uuid::Uuid;

use super::parser::{parse_http_request, parse_http_resonse};
#[derive(Clone)]
pub struct ConsoleHandler {
    tx: Sender<Bytes>,
}

impl ConsoleHandler {
    pub fn new(tx: Sender<Bytes>) -> Self {
        Self { tx }
    }
    pub fn init_transmitter(&self) -> (RequestTransmitter, ResponseTransmitter) {
        let uuid = Uuid::new_v4();
        (
            RequestTransmitter {
                id: uuid,
                tx: self.tx.clone(),
            },
            ResponseTransmitter {
                id: uuid,
                tx: self.tx.clone(),
            },
        )
    }
}

#[derive(Debug)]
pub struct RequestTransmitter {
    id: Uuid,
    tx: Sender<Bytes>,
}

#[async_trait]
pub trait Transmitter {
    async fn send(&self, data: Vec<u8>, req_count: i16) -> Result<bool>;
}
// unsafe impl Send for ResponseTransmitter {}
#[derive(Debug)]
pub struct ResponseTransmitter {
    id: Uuid,
    tx: Sender<Bytes>,
}

#[async_trait]
impl Transmitter for ResponseTransmitter {
    async fn send(&self, data: Vec<u8>, req_count: i16) -> Result<bool> {
        let mut data = parse_http_resonse(self.id.to_string(), data)?;
        let content_length = data.headers.get("Content-Length").map(|v| v[0].clone());
        if content_length.is_some() && data.body.is_empty() {
            // if content length is present and body is empty then the response might be segmented so we return true
            return Ok(true);
        }

        if content_length.is_some() && content_length.unwrap().parse::<i64>().unwrap() > 65536 {
            data.body = "body too large".to_string();
        }
        data.request_id = format!("{}-{:0>5}", self.id, req_count);
        self.tx
            .send(to_bytes(format!("data:{}\n\n", serde_json::to_string(&data)?)).await?)?;
        Ok(false)
    }
}
#[async_trait]
impl Transmitter for RequestTransmitter {
    async fn send(&self, data: Vec<u8>, req_count: i16) -> Result<bool> {
        let mut data = parse_http_request(self.id.to_string(), data)?;
        let content_length = data.headers.get("Content-Length").map(|v| v[0].clone());
        if content_length.is_some() && data.body.is_empty() {
            // if content length is present and body is empty then the response might be segmented so we return true
            return Ok(true);
        }

        if content_length.is_some() && content_length.unwrap().parse::<i64>().unwrap() > 65536 {
            data.body = "body too large".to_string();
        }

        data.id = format!("{}-{:0>5}", self.id, req_count);

        self.tx
            .send(to_bytes(format!("data:{}\n\n", serde_json::to_string(&data)?)).await?)?;
        Ok(false)
    }
}

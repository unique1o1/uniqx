use anyhow::Result;
use serde::Serialize;
use std::{collections::HashMap, vec};

#[derive(Serialize)]
pub struct ConsoleRequest {
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "method")]
    method: String,
    #[serde(rename = "url")]
    url: String,
    #[serde(rename = "body")]
    pub body: Vec<u8>,
    #[serde(rename = "headers")]
    pub headers: HashMap<String, Vec<String>>,
}

#[derive(Serialize)]
pub struct ConsoleResponse {
    #[serde(rename = "request_id")]
    pub request_id: String,
    #[serde(rename = "status")]
    status: u16,
    #[serde(rename = "headers")]
    pub headers: HashMap<String, Vec<String>>,
    #[serde(rename = "body")]
    pub body: Vec<u8>,
}
pub fn parse_http_resonse(id: String, data: Vec<u8>) -> Result<ConsoleResponse> {
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut res = httparse::Response::new(&mut headers);
    let status = res.parse(&data)?; // assuming that the response is complete

    let offset = status.unwrap();
    let headers: HashMap<String, Vec<String>> = res
        .headers
        .iter()
        .map(|h| {
            (
                h.name.to_string(),
                vec![std::str::from_utf8(h.value).unwrap().to_string()],
            )
        })
        .collect();
    Ok(ConsoleResponse {
        body: data[offset..].into(),
        headers,
        request_id: id.to_string(),
        status: res.code.unwrap(),
    })
}
pub fn parse_http_request(id: String, data: Vec<u8>) -> Result<ConsoleRequest> {
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = httparse::Request::new(&mut headers);

    let status = req.parse(&data)?; // assuming that the response is complete
    if status.is_partial() {
        return Err(anyhow::anyhow!("is partial"));
    }
    let offset = status.unwrap();
    let headers: HashMap<String, Vec<String>> = req
        .headers
        .iter()
        .map(|h| {
            (
                h.name.to_string(),
                vec![std::str::from_utf8(h.value).unwrap().to_string()],
            )
        })
        .collect();
    Ok(ConsoleRequest {
        body: data[offset..].into(),
        headers,
        id,
        method: req.method.unwrap().to_string(),
        url: req.path.unwrap().to_string(),
    })
}

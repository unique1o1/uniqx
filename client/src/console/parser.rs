use serde::Serialize;
use std::collections::HashMap;
use tracing::info;

use anyhow::Result;

pub fn str_from_u8_nul_utf8(utf8_src: &[u8]) -> &str {
    let nul_range_end = utf8_src
        .iter()
        .position(|&c| c == b'\0')
        .unwrap_or(utf8_src.len()); // default to length if no `\0` present
    std::str::from_utf8(&utf8_src[0..nul_range_end]).unwrap_or("unable to parse utf8")
}
#[derive(Serialize)]
pub struct ConsoleRequest {
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "method")]
    method: String,
    #[serde(rename = "url")]
    url: String,
    #[serde(rename = "body")]
    pub body: String,
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
    pub body: String,
}
pub fn parse_http_resonse(id: String, data: Vec<u8>) -> Result<ConsoleResponse> {
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut res = httparse::Response::new(&mut headers);
    let status = res.parse(&data)?; // assuming that the response is complete
                                    // if status.is_partial() {
                                    //     info!("is partial: _> {:?}", str_from_u8_nul_utf8(&data));
                                    //     return Err(anyhow::anyhow!("is partial"));
                                    // }
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
        body: str_from_u8_nul_utf8(&data[offset..]).to_string(),
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
        info!("is partial: _> {:?}", str_from_u8_nul_utf8(&data));
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
        body: str_from_u8_nul_utf8(&data[offset..]).to_string(),
        headers,
        id,
        method: req.method.unwrap().to_string(),
        url: req.path.unwrap().to_string(),
    })
}

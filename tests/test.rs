use std::time::Duration;

use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
use anyhow::Result;
use client::uniqx::UniqxClient;
use lazy_static::lazy_static;
use server::uniqx::UniqxServer;
use shared::utils::validate_subdomain;
use shared::SERVER_PORT;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::time;
lazy_static! {
    /// Guard to make sure that tests are run serially, not concurrently.
    static ref SERIAL_GUARD: Mutex<()> = Mutex::new(());
}

/// Spawn the server, giving some time for the control port TcpListener to start.
async fn spawn_server() {
    tokio::time::sleep(Duration::from_millis(50)).await;
    tokio::spawn(UniqxServer::new("localhost".to_owned(), 65454).start());
}

/// Spawns a client with randomly assigned ports, returning the listener and remote address.
async fn spawn_http_client() -> Result<(SocketAddr, SocketAddr)> {
    const LOCAL_POST: u16 = 65432;
    let client = UniqxClient::new(
        shared::Protocol::HTTP,
        LOCAL_POST,
        None,
        "localhost".to_owned(),
        "test".to_owned(),
        "localhost".to_owned(),
        false,
    )
    .await
    .unwrap();
    tokio::spawn(client.start());
    let local_http_addr: SocketAddr = ([127, 0, 0, 1], LOCAL_POST).into();
    let remote_addr: SocketAddr = ([127, 0, 0, 1], 65454).into();
    tokio::time::sleep(Duration::from_millis(100)).await;
    Ok((remote_addr, local_http_addr))
}
/// Spawns a client with randomly assigned ports, returning the listener and remote address.
async fn spawn_tcp_client() -> Result<(TcpListener, SocketAddr)> {
    let listener = TcpListener::bind(("0.0.0.0", 0)).await?;
    let local_port = listener.local_addr()?.port();
    let remote_port = 64878;
    let client = UniqxClient::new(
        shared::Protocol::TCP,
        local_port,
        Some(remote_port),
        "localhost".to_owned(),
        "test".to_owned(),
        "localhost".to_owned(),
        false,
    )
    .await
    .unwrap();
    tokio::spawn(client.start());
    let remote_addr: SocketAddr = ([127, 0, 0, 1], remote_port).into();
    tokio::time::sleep(Duration::from_millis(100)).await;
    Ok((listener, remote_addr))
}

#[tokio::test]
async fn tcp_proxy() -> Result<()> {
    let _guard = SERIAL_GUARD.lock().await;

    // initialize().await;
    spawn_server().await;
    let (listener, addr) = spawn_tcp_client().await?;

    tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut buf = [0u8; 11];
        stream.read_exact(&mut buf).await?;
        assert_eq!(&buf, b"hello world");
        stream.write_all(b"I can send a message too!").await?;
        anyhow::Ok(())
    });

    // let mut stream = TcpStream::connect(addr).await;
    let mut stream = TcpStream::connect(addr).await?;

    stream.write_all(b"hello world").await?;
    let mut buf = [0u8; 25];
    stream.read_exact(&mut buf).await.unwrap();
    assert_eq!(&buf, b"I can send a message too!");
    // Ensure that the client end of the stream is closed now.
    assert_eq!(stream.read(&mut buf).await?, 0);

    Ok(())
}
#[tokio::test]
async fn very_long_frame() -> Result<()> {
    let _guard = SERIAL_GUARD.lock().await;
    spawn_server().await;
    let mut attacker = TcpStream::connect(("localhost", SERVER_PORT)).await?;

    // Slowly send a very long frame.
    for _ in 0..10 {
        let result = attacker.write_all(&[42u8; 100000]).await;
        if result.is_err() {
            return Ok(());
        }
        time::sleep(Duration::from_millis(10)).await;
    }
    panic!("did not exit after a 1 MB frame");
}
#[test]
fn test_validate_subdomain() {
    // Valid subdomain
    assert!(validate_subdomain("example").is_ok());

    // Subdomain too short
    assert!(validate_subdomain("ex").is_err());

    // Subdomain too long
    assert!(validate_subdomain("this-subdomain-name-is-way-way-too-long").is_err());

    // Subdomain contains invalid character
    assert!(validate_subdomain("example!").is_err());

    // Subdomain is in deny list
    assert!(validate_subdomain("www").is_err());
}
pub fn str_from_u8_nul_utf8(utf8_src: &[u8]) -> &str {
    let nul_range_end = utf8_src
        .iter()
        .position(|&c| c == b'\0')
        .unwrap_or(utf8_src.len()); // default to length if no `\0` present
    std::str::from_utf8(&utf8_src[0..nul_range_end]).unwrap_or("unable to parse utf8")
}
#[tokio::test]
async fn http_proxy() -> Result<()> {
    let _guard = SERIAL_GUARD.lock().await;

    spawn_server().await;

    let (remote_addr, local_addr) = spawn_http_client().await?;

    let listner = HttpServer::new(move || {
        App::new().route(
            "/test",
            web::get().to(|req: HttpRequest| async move {
                println!("asdfasdfasdfas");
                assert_eq!(req.query_string(), "foo=bar");
                HttpResponse::Ok().body("Hello world!")
            }),
        )
    })
    .bind(local_addr)
    .unwrap();
    tokio::spawn(listner.disable_signals().run());

    let mut stream = TcpStream::connect(remote_addr).await.unwrap();

    let request = "GET /test?foo=bar HTTP/1.1\r\n\
                   Host: test.localhost:65454\r\n\
                   Connection: close\r\n\
                   \r\n";

    stream.write_all(request.as_bytes()).await?;
    let mut buf = [0u8; 1024];

    let _ = stream.read(&mut buf).await?;
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut res = httparse::Response::new(&mut headers);
    let status = res.parse(&buf).unwrap(); // assuming that the response is complete
    let offset = status.unwrap();
    assert_eq!(str_from_u8_nul_utf8(buf[offset..].into()), "Hello world!");
    Ok(())
}

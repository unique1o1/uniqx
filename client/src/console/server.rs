use actix_web::{
    get,
    middleware::Logger,
    web::{self, Bytes},
    App, HttpResponse, HttpServer, Responder,
};

use tokio::sync::broadcast::Sender;
use tokio_stream::wrappers::BroadcastStream;

use super::handler::ConsoleHandler;

#[get("/events")]
async fn event(data: web::Data<Sender<Bytes>>) -> impl Responder {
    let rx = BroadcastStream::new(data.subscribe());

    HttpResponse::Ok()
        .content_type("text/event-stream") // .append_header(("Cache-Control", "no-cache"))
        .append_header(("Connection", "keep-alive"))
        .streaming(rx)
}

async fn static_handler(file: &str, context_type: &str) -> impl Responder {
    HttpResponse::Ok()
        .content_type(context_type)
        .body(file.to_string())
}

pub async fn start() -> ConsoleHandler {
    let (tx, _) = tokio::sync::broadcast::channel(100);
    let data = web::Data::new(tx.clone());
    let listner = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(data.clone())
            .service(event)
            .route(
                "/",
                web::get().to(|| static_handler(include_str!("static/index.html"), "text/html")),
            )
            .route(
                "/script.js",
                web::get()
                    .to(|| static_handler(include_str!("static/script.js"), "text/javascript")),
            )
            .route(
                "/style.css",
                web::get().to(|| static_handler(include_str!("static/style.css"), "text/css")),
            )
    })
    .bind(("127.0.0.1", 0))
    .unwrap();

    let port = listner.addrs()[0].port();
    tokio::spawn(listner.disable_signals().run());
    ConsoleHandler::new(tx, port)
}

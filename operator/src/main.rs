use std::io;

use actix_web::{
    get, middleware, web::Data, App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use controller::{Config, State};
use dotenv::dotenv;
use prometheus::{Encoder, TextEncoder};

#[get("/metrics")]
async fn metrics(c: Data<State>, _req: HttpRequest) -> impl Responder {
    let metrics = c.metrics();
    let encoder = TextEncoder::new();
    let mut buffer = vec![];
    encoder.encode(&metrics, &mut buffer).unwrap();
    HttpResponse::Ok().body(buffer)
}

#[get("/health")]
async fn health(_: HttpRequest) -> impl Responder {
    HttpResponse::Ok().json("healthy")
}

#[tokio::main]
async fn main() -> io::Result<()> {
    dotenv().ok();

    let state = State::default();
    let config = Config::default();

    let controller = controller::run(state.clone(), config);

    let addr = std::env::var("ADDR").unwrap_or("0.0.0.0:8080".into());

    let server = HttpServer::new(move || {
        App::new()
            .app_data(Data::new(state.clone()))
            .wrap(middleware::Logger::default())
            .service(health)
            .service(metrics)
    })
    .bind(addr)?;

    tokio::join!(controller, server.run()).1?;

    Ok(())
}

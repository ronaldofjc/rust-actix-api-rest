mod create_user;
mod error;
mod health;
mod repository;
mod user;
mod v1;

use crate::error::Error;
use crate::repository::MemoryRepository;
use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::Arc;
use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    // init tracing subscriber
    let tracing = tracing_subscriber::fmt()
        .with_span_events(FmtSpan::ENTER | FmtSpan::CLOSE)
        .with_timer(tracing_subscriber::fmt::time::ChronoUtc::rfc3339())
        .with_env_filter(EnvFilter::from_default_env());

    if cfg!(debug_assertions) {
        tracing.pretty().init();
    } else {
        tracing.json().init();
    }

    let port = std::env::var("PORT").unwrap_or("8090".to_string());
    let address = format!("127.0.0.1:{}", port);

    tracing::debug!("Starting our server at {}", address);
    let thread_counter = Arc::new(AtomicU16::new(1));
    let repo = web::Data::new(MemoryRepository::default());

    HttpServer::new(move || {
        let thread_index = thread_counter.fetch_add(1, Ordering::SeqCst);
        tracing::trace!("Starting thread {}", thread_index);

        App::new()
            .wrap(Cors::default().supports_credentials())
            .data(thread_index)
            .app_data(repo.clone())
            .configure(v1::service::<MemoryRepository>)
            .configure(health::service)
    })
    .bind(&address)
    .unwrap_or_else(|err| {
        panic!(
            "🔥🔥🔥 Couldn't start the server in port {}: {:?}",
            port, err
        )
    })
    .run()
    .await
}

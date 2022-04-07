mod create_user;
mod error;
mod health;
mod repository;
mod user;
mod v1;

use crate::error::Error;
use crate::repository::{PostgresRepository};
//use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::Arc;
use tracing_subscriber::{EnvFilter};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    // init tracing subscriber
    let tracing = tracing_subscriber::fmt()
        .with_timer(tracing_subscriber::fmt::time::UtcTime::rfc_3339())
        .with_env_filter(EnvFilter::from_default_env());

    if cfg!(debug_assertions) {
        tracing.pretty().init();
    } else {
        tracing.json().init();
    }

    let port = std::env::var("PORT").unwrap_or("8090".to_string());
    let address = format!("127.0.0.1:{}", port);

    tracing::info!("Starting server at {}", address);
    let thread_counter = Arc::new(AtomicU16::new(1));
    let pos_repo = PostgresRepository::from_env().await.expect("Repository initialize error");
    let repo = web::Data::new(pos_repo);

    HttpServer::new(move || {
        let thread_index = thread_counter.fetch_add(1, Ordering::SeqCst);
        tracing::trace!("Starting thread {}", thread_index);

        App::new()
            //.wrap(Cors::default().supports_credentials())
            .app_data(web::Data::new(thread_index))
            .app_data(repo.clone())
            .configure(v1::service::<PostgresRepository>)
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

#[cfg(test)]
mod main {
    use actix_web::{App, web};
    use actix_web::http::StatusCode;
    use crate::health::service;

    #[actix_rt::test]
    async fn app_main_integration_test() {
        let app = App::new().app_data(web::Data::new(5u16)).configure(service);
        let mut app = actix_web::test::init_service(app).await;
        let req = actix_web::test::TestRequest::get()
            .uri("/health")
            .to_request();
        let res = actix_web::test::call_service(&mut app, req).await;
        assert!(res.status().is_success());
        assert_eq!(res.status(), StatusCode::OK);
        let data = res
            .headers()
            .get("thread-id")
            .map(|h| h.to_str().ok())
            .flatten();
        assert_eq!(data, Some("5"))
    }
}

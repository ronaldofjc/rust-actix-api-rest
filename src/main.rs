mod user;
mod repository;
mod error;
mod health;
mod v1;
mod create_user;

use std::sync::Arc;
use std::sync::atomic::{AtomicU16, Ordering};
use actix_web::{App, HttpServer, web};
use crate::error::Error;
use crate::repository::{MemoryRepository};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let port = std::env::var("PORT").unwrap_or("8080".to_string());
    let address = format!("127.0.0.1:{}", port);

    println!("Starting server");
    let thread_counter = Arc::new(AtomicU16::new(1));
    let repo = web::Data::new(MemoryRepository::default());

    HttpServer::new(move || {
        let thread_index = thread_counter.fetch_add(1, Ordering::SeqCst);
        println!("Starting Thread {}", thread_index);

        App::new()
            .data(thread_index)
            .app_data(repo.clone())
            .configure(v1::service::<MemoryRepository>)
            .configure(health::service)
    })
        .bind(&address)
        .unwrap_or_else(|err| panic!("🔥🔥🔥 Couldn't start the server in port {}: {:?}", port, err))
        .run()
        .await
}
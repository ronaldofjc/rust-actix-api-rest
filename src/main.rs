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
    use httpmock::prelude::*;
    use isahc::{prelude::*, get};
    use crate::health::service;
    use crate::user::{create_test_user, User};

    const HTTP: &str = "http://";
    const HTTP_GET_USER: &str = "/v1/user/71802ecd-4eb3-4381-af7e-f737e3a35d5d";
    const USER_NAME: &str = "Meu nome";

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

    #[actix_rt::test]
    async fn http_rest_get_all_users_test() {
        let server = MockServer::start();
        let users = vec![create_test_user(uuid::Uuid::new_v4(), USER_NAME.to_string(), (1977, 03, 10))];
        let m = server.mock(|when, then| {
            when.method(GET)
                .path("/v1/user");
            then.status(200).json_body_obj(&users);
        });

        let mut response = get(&format!("{}{}/v1/user", HTTP, server.address())).unwrap();
        let users: Vec<User> = serde_json::from_str(&response.text().unwrap()).expect("cannot deserialize JSON");

        m.assert();
        assert_eq!(response.status(), 200);
        assert_eq!(users.get(0).unwrap().name, USER_NAME)
    }

    #[actix_rt::test]
    async fn http_rest_get_user_test() {
        let server = MockServer::start();
        let user = create_test_user(uuid::Uuid::new_v4(), USER_NAME.to_string(), (1977, 03, 10));
        let m = server.mock(|when, then| {
            when.method(GET)
                .path(HTTP_GET_USER);
            then.status(200).json_body_obj(&user);
        });

        let mut response = get(&format!("{}{}{}", HTTP, server.address(), HTTP_GET_USER)).unwrap();
        let user: User = serde_json::from_str(&response.text().unwrap()).expect("cannot deserialize JSON");

        m.assert();
        assert_eq!(response.status(), 200);
        assert_eq!(user.name, USER_NAME)
    }
}

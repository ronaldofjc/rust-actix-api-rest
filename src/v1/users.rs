use actix_web::{HttpRequest, HttpResponse, web};
use actix_web::error::PathError;
use actix_web::web::{PathConfig, ServiceConfig};
use uuid::Uuid;
use crate::repository::Repository;

const PATH: &str = "/user";

pub fn service<R: Repository>(cfg: &mut ServiceConfig) {
    cfg.service(web::scope(PATH)
        .app_data(PathConfig::default().error_handler(path_config_handler))
        .route("/{user_id}", web::get().to(get::<R>))
    );
}

async fn get<R: Repository>(user_id: web::Path<Uuid>, repo: web::Data<R>) -> HttpResponse {
    match repo.get_user(&user_id) {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(err) => HttpResponse::NotFound().json(err)
    }
}

fn path_config_handler(err: PathError, _req: &HttpRequest) -> actix_web::Error {
    actix_web::error::ErrorBadRequest(err)
}

#[cfg(test)]
mod tests {
    use actix_web::http::StatusCode;
    use crate::MemoryRepository;
    use super::*;

    #[actix_rt::test]
    async fn get_user_service_with_success() {
        let res = get(web::Path::from(uuid::Uuid::parse_str("71802ecd-4eb3-4381-af7e-f737e3a35d5c").unwrap()),
              RepositoryInjection::new(MemoryRepository::default())
        ).await;
        assert!(res.status().is_success());
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[actix_rt::test]
    async fn get_user_service_with_error() {
        let res = get(web::Path::from(uuid::Uuid::parse_str("71802ecd-4eb3-4381-af7e-f737e3a35d5d").unwrap()),
                  RepositoryInjection::new(MemoryRepository::default())
        ).await;
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }
}
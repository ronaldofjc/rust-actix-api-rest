use actix_web::{HttpResponse, web};
use actix_web::web::ServiceConfig;
use uuid::Uuid;
use crate::{Error, RepositoryInjection};

const PATH: &str = "/user";

pub fn service(cfg: &mut ServiceConfig) {
    cfg.service(web::scope(PATH).route("/{user_id}", web::get().to(get)));
}

async fn get(user_id: web::Path<String>, repo: web::Data<RepositoryInjection>) -> HttpResponse {
    if let Ok(parsed_user_id) = Uuid::parse_str(&user_id) {
        match repo.get_user(&parsed_user_id) {
            Ok(user) => HttpResponse::Ok().json(user),
            Err(err) => HttpResponse::NotFound().json(err)
        }
    } else {
        HttpResponse::BadRequest().json(Error::new("Invalid UUID".to_string(), 400))
    }
}

#[cfg(test)]
mod tests {
    use actix_web::http::StatusCode;
    use crate::MemoryRepository;
    use super::*;

    #[actix_rt::test]
    async fn get_user_service_with_success() {
        let res = get(web::Path::from("71802ecd-4eb3-4381-af7e-f737e3a35d5c".to_string()),
                      web::Data::new(RepositoryInjection::new(MemoryRepository::default()))
        ).await;
        assert!(res.status().is_success());
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[actix_rt::test]
    async fn get_user_service_with_invalid_uuid() {
        let res = get(web::Path::from("71802ecd-4eb3-4381-af7e-f737e3a35d5cd".to_string()),
                      web::Data::new(RepositoryInjection::new(MemoryRepository::default()))
        ).await;
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }

    #[actix_rt::test]
    async fn get_user_service_with_error() {
        let res = get(web::Path::from("71802ecd-4eb3-4381-af7e-f737e3a35d5d".to_string()),
                      web::Data::new(RepositoryInjection::new(MemoryRepository::default()))
        ).await;
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }
}
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
    use super::*;
    use crate::user::User;
    use crate::{Error, MemoryRepository};
    use mockall::*;
    use mockall::predicate::*;

    mock! {
        CustomRepo {}
        impl Repository for CustomRepo {
            fn get_user(&self, user_id: &uuid::Uuid) -> Result<User, Error>;
        }
    }

    #[actix_rt::test]
    async fn get_user_service_with_success() {
        let user_id = uuid::Uuid::new_v4();
        let user_name = "Meu nome";

        let mut repo = MockCustomRepo::default();
        repo.expect_get_user().returning(move |id| {
            let mut user = User::new(user_name.to_string(), (1983, 08, 30));
            user.id = *id;
            Ok(user)
        });

        let mut result = get(web::Path::from(user_id), web::Data::new(repo)).await;
        let user = result
            .take_body()
            .as_ref()
            .map(|b| match b {
                actix_web::dev::Body::Bytes(x) => serde_json::from_slice::<'_, User>(x).ok(), _ => None,
            })
            .flatten()
            .unwrap();

        assert_eq!(user.id, user_id);
        assert_eq!(user.name, user_name);
    }

    #[actix_rt::test]
    async fn get_user_service_with_error() {
        let res = get(web::Path::from(uuid::Uuid::parse_str("71802ecd-4eb3-4381-af7e-f737e3a35d5d").unwrap()),
              web::Data::new(MemoryRepository::default())
        ).await;
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }
}
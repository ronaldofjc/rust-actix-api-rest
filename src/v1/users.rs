use actix_web::{HttpRequest, HttpResponse, web};
use actix_web::error::PathError;
use actix_web::web::{PathConfig, ServiceConfig};
use uuid::Uuid;
use crate::create_user::CreateUser;
use crate::repository::Repository;
use crate::user::User;

const PATH: &str = "/user";

pub fn service<R: Repository>(cfg: &mut ServiceConfig) {
    cfg.service(web::scope(PATH)
        .app_data(PathConfig::default().error_handler(path_config_handler))
        .route("", web::get().to(get_all::<R>))
        .route("/{user_id}", web::get().to(get::<R>))
        .route("", web::post().to(post::<R>))
        .route("", web::put().to(put::<R>))
        .route("/{user_id}", web::delete().to(delete::<R>))
    );
}

async fn get_all<R: Repository>(repo: web::Data<R>) -> HttpResponse {
    match repo.get_all().await {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(err) => HttpResponse::NotFound().json(err)
    }
}

async fn get<R: Repository>(user_id: web::Path<Uuid>, repo: web::Data<R>) -> HttpResponse {
    match repo.get_user(&user_id).await {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(err) => HttpResponse::NotFound().json(err)
    }
}

async fn post<R: Repository>(user: web::Json<CreateUser>, repo: web::Data<R>) -> HttpResponse {
    match repo.create_user(&user).await {
        Ok(user) => HttpResponse::Created().json(user),
        Err(err) => HttpResponse::UnprocessableEntity().json(err)
    }
}

async fn put<R: Repository>(user: web::Json<User>, repo: web::Data<R>) -> HttpResponse {
    match repo.update_user(&user).await {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(err) => HttpResponse::UnprocessableEntity().json(err)
    }
}

async fn delete<R: Repository>(user_id: web::Path<Uuid>, repo: web::Data<R>) -> HttpResponse {
    match repo.delete_user(&user_id).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(err) => HttpResponse::InternalServerError().json(err)
    }
}

fn path_config_handler(err: PathError, _req: &HttpRequest) -> actix_web::Error {
    actix_web::error::ErrorBadRequest(err)
}

#[cfg(test)]
mod tests {
    use actix_web::http::StatusCode;
    use chrono::{NaiveDate, Utc};
    use super::*;
    use crate::user::{CustomData, User};
    use crate::{Error, MemoryRepository};
    use crate::create_user::{CreateUser, CustomData as OtherCustomData};
    use mockall::*;
    use mockall::predicate::*;

    mock! {
        CustomRepo {}
        impl Repository for CustomRepo {
            fn get_all(&self) -> Result<Vec<User>, Error>;
            fn get_user(&self, user_id: &uuid::Uuid) -> Result<User, Error>;
            fn create_user(&self, user: &CreateUser) -> Result<User, Error>;
            fn update_user(&self, user: &User) -> Result<User, Error>;
            fn delete_user(&self, user_id: &uuid::Uuid) -> Result<Uuid, Error>;
            fn get_user_by_email(&self, user_email: &String) -> Result<User, Error>;
        }
    }

    pub fn create_test_user(id: uuid::Uuid, name: String, birth_date_ymd: (i32, u32, u32)) -> User {
        let (year, month, day) = birth_date_ymd;
        User {
            id,
            name,
            email: "teste@teste.com".to_string(),
            birth_date: NaiveDate::from_ymd(year, month, day),
            custom_data: CustomData { random: 1 },
            created_at: Some(Utc::now()),
            updated_at: None,
        }
    }

    pub fn create_test_user_request(name: String, birth_date_ymd: (i32, u32, u32)) -> CreateUser {
        let (year, month, day) = birth_date_ymd;
        CreateUser {
            name,
            email: "teste@teste.com".to_string(),
            birth_date: NaiveDate::from_ymd(year, month, day),
            custom_data: OtherCustomData { random: 1 },
            created_at: Some(Utc::now()),
            updated_at: None,
        }
    }

    #[actix_rt::test]
    async fn get_works() {
        let user_id = uuid::Uuid::new_v4();
        let user_name = "Meu nome";

        let mut repo = MockCustomRepo::default();
        repo.expect_get_user().returning(move |id| {
            let user = create_test_user(*id, user_name.to_string(), (1977, 03, 10));
            Ok(user)
        });

        let mut result = get(web::Path::from(user_id), web::Data::new(repo)).await;

        let user = result
            .take_body()
            .as_ref()
            .map(|b| match b {
                actix_web::dev::Body::Bytes(x) => serde_json::from_slice::<'_, User>(x).ok(),
                _ => None,
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

    #[actix_rt::test]
    async fn create_works() {
        let user_id = uuid::Uuid::new_v4();
        let user_name = "Meu nome";
        let create_user = create_test_user_request(user_name.to_string(), (1977, 03, 10));

        let mut repo = MockCustomRepo::default();
        repo.expect_create_user()
            .returning(move |_user| {
                let new_user = create_test_user(user_id, user_name.to_string(), (1977, 03, 10));
                Ok(new_user)
            });

        let mut result = post(web::Json(create_user), web::Data::new(repo)).await;

        let user = result
            .take_body()
            .as_ref()
            .map(|b| match b {
                actix_web::dev::Body::Bytes(x) => serde_json::from_slice::<'_, User>(x).ok(),
                _ => None,
            })
            .flatten()
            .unwrap();

        assert_eq!(user.id, user_id);
        assert_eq!(user.name, user_name);
    }

    #[actix_rt::test]
    async fn update_works() {
        let user_id = uuid::Uuid::new_v4();
        let user_name = "Meu nome";
        let new_user = create_test_user(user_id, user_name.to_string(), (1977, 03, 10));

        let mut repo = MockCustomRepo::default();
        repo.expect_update_user()
            .returning(|user| Ok(user.to_owned()));

        let mut result = put(web::Json(new_user), web::Data::new(repo)).await;

        let user = result
            .take_body()
            .as_ref()
            .map(|b| match b {
                actix_web::dev::Body::Bytes(x) => serde_json::from_slice::<'_, User>(x).ok(),
                _ => None,
            })
            .flatten()
            .unwrap();

        assert_eq!(user.id, user_id);
        assert_eq!(user.name, user_name);
    }

    #[actix_rt::test]
    async fn delete_works() {
        let user_id = uuid::Uuid::new_v4();

        let mut repo = MockCustomRepo::default();
        repo.expect_delete_user().returning(|id| Ok(id.to_owned()));

        let result = delete(web::Path::from(user_id), web::Data::new(repo)).await;

        assert_eq!(result.status(), StatusCode::NO_CONTENT);
    }
}
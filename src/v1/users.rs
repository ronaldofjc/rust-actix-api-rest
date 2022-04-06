use crate::create_user::CreateUser;
use crate::repository::Repository;
use crate::user::User;
use actix_web::error::PathError;
use actix_web::web::{PathConfig, ServiceConfig, self};
use actix_web::{HttpRequest, HttpResponse};
use uuid::Uuid;

const PATH: &str = "/user";

pub fn service<R: Repository>(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope(PATH)
            .app_data(PathConfig::default().error_handler(path_config_handler))
            .route("", web::get().to(get_all::<R>))
            .route("/{user_id}", web::get().to(get::<R>))
            .route("", web::post().to(post::<R>))
            .route("", web::put().to(put::<R>))
            .route("/{user_id}", web::delete().to(delete::<R>)),
    );
}

async fn get_all<R: Repository>(repo: web::Data<R>) -> HttpResponse {
    match repo.get_all().await {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(err) => HttpResponse::BadGateway().json(err),
    }
}

async fn get<R: Repository>(user_id: web::Path<Uuid>, repo: web::Data<R>) -> HttpResponse {
    match repo.get_user(&user_id).await {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

async fn post<R: Repository>(user: web::Json<CreateUser>, repo: web::Data<R>) -> HttpResponse {
    match repo.create_user(&user).await {
        Ok(user) => HttpResponse::Created().json(user),
        Err(err) => HttpResponse::UnprocessableEntity().json(err),
    }
}

async fn put<R: Repository>(user: web::Json<User>, repo: web::Data<R>) -> HttpResponse {
    match repo.update_user(&user).await {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(err) => HttpResponse::UnprocessableEntity().json(err),
    }
}

async fn delete<R: Repository>(user_id: web::Path<Uuid>, repo: web::Data<R>) -> HttpResponse {
    match repo.delete_user(&user_id).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

fn path_config_handler(err: PathError, _req: &HttpRequest) -> actix_web::Error {
    actix_web::error::ErrorBadRequest(err)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_user::{CreateUser, CustomData as OtherCustomData};
    use crate::user::{CustomData, User};
    use crate::{repository::MockRepository, Error};
    use actix_web::http::StatusCode;
    use chrono::{NaiveDate, Utc};

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
    async fn get_all_with_success() {
        let user_id = uuid::Uuid::new_v4();
        let user_name = "Meu nome";

        let mut repo = MockRepository::default();
        repo.expect_get_all().returning(move || {
            let users = vec![create_test_user(user_id, user_name.to_string(), (1977, 03, 10))];
            Ok(users)
        });

        let result = get_all(web::Data::new(repo)).await;
        assert_eq!(result.status(), StatusCode::OK);
    }

    #[actix_rt::test]
    async fn get_all_with_error() {
        let mut repo = MockRepository::default();
        repo.expect_get_all().returning(move || Err(Error::new("error".to_string(), 502)));

        let result = get_all(web::Data::new(repo)).await;
        assert_eq!(result.status(), StatusCode::BAD_GATEWAY);
    }

    #[actix_rt::test]
    async fn get_user_with_success() {
        let user_id = uuid::Uuid::new_v4();
        let user_name = "Meu nome";

        let mut repo = MockRepository::default();
        repo.expect_get_user().returning(move |id| {
            let user = create_test_user(*id, user_name.to_string(), (1977, 03, 10));
            Ok(user)
        });

        let result = get(web::Path::from(user_id), web::Data::new(repo)).await;

        assert_eq!(result.status(), StatusCode::OK);
    }

    #[actix_rt::test]
    async fn get_user_with_error() {
        let user_id = uuid::Uuid::parse_str("71802ecd-4eb3-4381-af7e-f737e3a35d5d");
        let mut repo = MockRepository::default();
        repo.expect_get_user()
            .returning(move |_id| Err(Error::new("error".to_string(), 404)));
        let res = get(web::Path::from(user_id.unwrap()), web::Data::new(repo)).await;
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }

    #[actix_rt::test]
    async fn create_with_success() {
        let user_id = uuid::Uuid::new_v4();
        let user_name = "Meu nome";
        let create_user = create_test_user_request(user_name.to_string(), (1977, 03, 10));

        let mut repo = MockRepository::default();
        repo.expect_create_user().returning(move |_user| {
            let new_user = create_test_user(user_id, user_name.to_string(), (1977, 03, 10));
            Ok(new_user)
        });

        let result = post(web::Json(create_user), web::Data::new(repo)).await;

        assert_eq!(result.status(), StatusCode::CREATED);
    }

    #[actix_rt::test]
    async fn create_with_error() {
        let user_name = "Meu nome";
        let create_user = create_test_user_request(user_name.to_string(), (1977, 03, 10));

        let mut repo = MockRepository::default();
        repo.expect_create_user().returning(move |_user| Err(Error::new("error".to_string(), 422)));

        let result = post(web::Json(create_user), web::Data::new(repo)).await;
        assert_eq!(result.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[actix_rt::test]
    async fn update_with_success() {
        let user_id = uuid::Uuid::new_v4();
        let user_name = "Meu nome";
        let new_user = create_test_user(user_id, user_name.to_string(), (1977, 03, 10));

        let mut repo = MockRepository::default();
        repo.expect_update_user().returning(|user| Ok(user.to_owned()));

        let result = put(web::Json(new_user), web::Data::new(repo)).await;
        assert_eq!(result.status(), StatusCode::OK);
    }

    #[actix_rt::test]
    async fn update_with_error() {
        let user_id = uuid::Uuid::new_v4();
        let user_name = "Meu nome";
        let new_user = create_test_user(user_id, user_name.to_string(), (1977, 03, 10));

        let mut repo = MockRepository::default();
        repo.expect_update_user().returning(|_user| Err(Error::new("error".to_string(), 422)));

        let result = put(web::Json(new_user), web::Data::new(repo)).await;
        assert_eq!(result.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[actix_rt::test]
    async fn delete_with_success() {
        let user_id = uuid::Uuid::new_v4();

        let mut repo = MockRepository::default();
        repo.expect_delete_user().returning(|id| Ok(id.to_owned()));

        let result = delete(web::Path::from(user_id), web::Data::new(repo)).await;
        assert_eq!(result.status(), StatusCode::NO_CONTENT);
    }

    #[actix_rt::test]
    async fn delete_with_error() {
        let user_id = uuid::Uuid::new_v4();

        let mut repo = MockRepository::default();
        repo.expect_delete_user().returning(|_id| Err(Error::new("error".to_string(), 422)));

        let result = delete(web::Path::from(user_id), web::Data::new(repo)).await;
        assert_eq!(result.status(), StatusCode::NOT_FOUND);
    }
}

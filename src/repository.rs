use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

use crate::create_user::CreateUser;
use crate::user::{User};
use crate::Error;

pub type RepositoryResult<T> = Result<T, Error>;
pub type RepositoryResultList<T> = Result<Vec<T>, Error>;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait Repository: Send + Sync + 'static {
    async fn get_all(&self) -> RepositoryResultList<User>;
    async fn get_user(&self, user_id: &Uuid) -> RepositoryResult<User>;
    async fn get_user_by_email(&self, user_email: &String) -> RepositoryResult<User>;
    async fn create_user(&self, user: &CreateUser) -> RepositoryResult<User>;
    async fn update_user(&self, user: &User) -> RepositoryResult<User>;
    async fn delete_user(&self, user_id: &Uuid) -> RepositoryResult<Uuid>;
}

pub struct PostgresRepository {
    pool: sqlx::PgPool,
}

impl PostgresRepository {
    pub async fn from_env() -> sqlx::Result<Self> {
        let conn_str =
            std::env::var("DATABASE_URL").map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;
        let pool = sqlx::PgPool::connect(&conn_str).await?;
        Ok(Self { pool })
    }
}

#[async_trait]
impl Repository for PostgresRepository {
    async fn get_all(&self) -> RepositoryResultList<User> {
        let users = sqlx::query_as::<_, User>(
            "SELECT * FROM users",
        ).fetch_all(&self.pool)
        .await;

        tracing::info!("Repository returning {} users", users.as_ref().unwrap().len());

        users.map_err(|e| {
            tracing::error!("Error on get all users, error: {:?}", e);
            Error::new("Error on get all users".to_string(), 502)
        })
    }

    async fn get_user(&self, user_id: &uuid::Uuid) -> RepositoryResult<User> {
        let result = sqlx::query_as::<_, User>(
            "SELECT id, name, email, birth_date, custom_data, created_at, updated_at FROM users WHERE id = $1",
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await;

        tracing::info!("Repository returning {:?}", result.as_ref().to_owned().unwrap());

        result.map_err(|e| {
            tracing::error!("{:?}", e);
            Error::new("Invalid Uuid".to_string(), 404)
        })
    }

    async fn get_user_by_email(&self, user_email: &String) -> RepositoryResult<User> {
        let result = sqlx::query_as::<_, User>(
            "SELECT id, name, email, birth_date, custom_data, created_at, updated_at FROM users WHERE email = $1",
        )
        .bind(user_email)
        .fetch_one(&self.pool)
        .await;

        result.map_err(|e| {
            tracing::error!("Error on get user by email {}. Error: {:?}", user_email, e);
            Error::new("Error on get user by email".to_string(), 400)
        })
    }

    async fn create_user(&self, user: &CreateUser) -> RepositoryResult<User> {
        if let Ok(_old_user) = self.get_user_by_email(&user.email).await {
            tracing::warn!("User with email {} already exists", user.email);
            return Result::Err(Error::new("This user already exists".to_string(), 422));
        }

        let result = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, name, email, birth_date, custom_data, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, name, email, birth_date, custom_data, created_at, updated_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(&user.name)
        .bind(&user.email)
        .bind(&user.birth_date)
        .bind(&user.custom_data)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await;

        tracing::info!("User with email {} was created", user.email);

        result.map_err(|e| {
            tracing::error!("{:?}", e);
            Error::new("This user already exists".to_string(), 400)
        })
    }

    async fn update_user(&self, user: &User) -> RepositoryResult<User> {
        if let Ok(_old_user) = self.get_user(&user.id).await {
            if let Ok(database_user) = self.get_user_by_email(&user.email).await {
                if database_user.email != user.email {
                    tracing::warn!("User with email {} already exists", user.email);
                    return Result::Err(Error::new("This user email already exists".to_string(), 422));
                }
            }

            let result = sqlx::query_as::<_, User>(
                r#"
                UPDATE users
                SET custom_data = $1, updated_at = $2, name = $3, email = $4
                WHERE id = $5
                RETURNING id, name, email, birth_date, custom_data, created_at, updated_at
                "#,
            )
            .bind(&user.custom_data)
            .bind(Utc::now())
            .bind(&user.name)
            .bind(&user.email)
            .bind(&user.id)
            .fetch_one(&self.pool)
            .await;

            tracing::info!("User with email {} was updated", user.email);

            result.map_err(|e| {
                tracing::error!("{:?}", e);
                Error::new("Error on update user".to_string(), 502)
            })
        } else {
            tracing::error!("User with id {} not found", user.id);
            Err(Error::new("This user does not exist".to_string(), 400))
        }
    }

    async fn delete_user(&self, user_id: &Uuid) -> RepositoryResult<Uuid> {
        let result = sqlx::query_as::<_, User>(
            r#"
            DELETE FROM users
            WHERE id = $1
            RETURNING id, name, email, birth_date, custom_data, created_at, updated_at
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await;

        tracing::info!("User with id {} was removed", user_id);

        result.map(|u| u.id).map_err(|e| {
            tracing::error!("{:?}", e);
            Error::new("This user does not exist".to_string(), 404)
        })
    }
}

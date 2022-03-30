use std::sync::RwLock;
use chrono::Utc;
use uuid::Uuid;
use async_trait::async_trait;

use crate::create_user::CreateUser;
use crate::Error;
use crate::user::{CustomData, User};

const USER_ERROR: &str = "Get user error";

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

pub struct MemoryRepository {
    users: RwLock<Vec<User>>
}

impl Default for MemoryRepository {
    fn default() -> Self {
        Self {
            users: RwLock::new(vec![])
        }
    }
}

#[async_trait]
impl Repository for MemoryRepository {
    async fn get_all(&self) -> RepositoryResultList<User> {
        let users = self.users.read()
            .map_err(|_| Error::new(USER_ERROR.to_string(), 406))?;
        Ok(users.clone())
    }

    async fn get_user(&self, user_id: &uuid::Uuid) -> RepositoryResult<User> {
        let users = self.users.read()
            .map_err(|_| Error::new(USER_ERROR.to_string(), 406))?;
        users
            .iter()
            .find(|u| &u.id == user_id)
            .cloned()
            .ok_or_else(|| Error::new("Invalid Uuid".to_string(), 404))
    }

    async fn get_user_by_email(&self, user_email: &String) -> RepositoryResult<User> {
        let users = self.users.read()
            .map_err(|_| Error::new(USER_ERROR.to_string(), 406))?;
        users
            .iter()
            .find(|u| &u.email == user_email)
            .cloned()
            .ok_or_else(|| Error::new("User not found".to_string(), 404))
    }

    async fn create_user(&self, user: &CreateUser) -> RepositoryResult<User> {
        if let Ok(_old_user) = self.get_user_by_email(&user.email).await {
            return Result::Err(Error::new("This user already exists".to_string(), 400));
        }
        let create_user = user.to_owned();
        let new_user = User {
            id: Uuid::new_v4(),
            name: create_user.name,
            email: create_user.email,
            birth_date: create_user.birth_date,
            custom_data: CustomData {
                random: create_user.custom_data.random
            },
            created_at: Some(Utc::now()),
            updated_at: None
        };

        let mut users = self.users.write()
            .map_err(|_| Error::new(USER_ERROR.to_string(), 406))?;
        users.push(new_user.clone());
        Ok(new_user)
    }

    async fn update_user(&self, user: &User) -> RepositoryResult<User> {
        if let Ok(old_user) = self.get_user(&user.id).await {
            let mut updated_user = user.to_owned();
            updated_user.created_at = old_user.created_at;
            updated_user.updated_at = Some(Utc::now());
            let mut users = self.users.write().unwrap();
            users.retain(|x| x.id != user.id);
            users.push(updated_user.clone());
            Ok(updated_user)
        } else {
            Err(Error::new("This user does not exist".to_string(), 404))
        }
    }

    async fn delete_user(&self, user_id: &Uuid) -> RepositoryResult<Uuid> {
        let mut users = self.users.write()
            .map_err(|_| Error::new(USER_ERROR.to_string(), 406))?;
        users.retain(|x| &x.id != user_id);
        Ok(user_id.to_owned())
    }
}

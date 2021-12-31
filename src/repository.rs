use std::sync::RwLock;
use chrono::Utc;
use uuid::Uuid;
use crate::Error;
use crate::user::User;

pub trait Repository: Send + Sync + 'static {
    fn get_user(&self, user_id: &Uuid) -> Result<User, Error>;
    fn create_user(&self, user: &User) -> Result<User, Error>;
    fn update_user(&self, user: &User) -> Result<User, Error>;
    fn delete_user(&self, user_id: &uuid::Uuid) -> Result<Uuid, Error>;
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

impl Repository for MemoryRepository {
    fn get_user(&self, user_id: &uuid::Uuid) -> Result<User, Error> {
        let users = self.users.read()
            .map_err(|_| Error::new("Unlock error".to_string(), 406))?;
        users
            .iter()
            .find(|u| &u.id == user_id)
            .cloned()
            .ok_or_else(|| Error::new("User not found".to_string(), 404))
    }

    fn create_user(&self, user: &User) -> Result<User, Error> {
        if self.get_user(&user.id).is_ok() {
            return Result::Err(Error::new("This user already exists".to_string(), 404));
        }
        let mut new_user = user.to_owned();
        new_user.created_at = Some(Utc::now());
        let mut users = self.users.write()
            .map_err(|_| Error::new("Unlock error".to_string(), 406))?;
        users.push(new_user.clone());
        Result::Ok(new_user)
    }

    fn update_user(&self, user: &User) -> Result<User, Error> {
        if self.get_user(&user.id).is_err() {
            return Result::Err(Error::new("This user does not exist".to_string(), 404));
        }
        let mut update_user = user.to_owned();
        update_user.updated_at = Some(Utc::now());
        let mut users = self.users.write()
            .map_err(|_| Error::new("Unlock error".to_string(), 406))?;
        users.retain(|u| u.id != user.id);
        users.push(update_user.clone());
        Result::Ok(update_user)
    }

    fn delete_user(&self, user_id: &Uuid) -> Result<Uuid, Error> {
        let mut users = self.users.write()
            .map_err(|_| Error::new("Unlock error".to_string(), 406))?;
        users.retain(|u| &u.id != user_id);
        Result::Ok(user_id.to_owned())
    }
}

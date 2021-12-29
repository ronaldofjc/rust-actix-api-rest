use std::ops::Deref;
use crate::Error;
use crate::user::User;

pub trait Repository: Send + Sync + 'static {
    fn get_user(&self, user_id: &uuid::Uuid) -> Result<User, Error>;
}

pub struct RepositoryInjection(Box<dyn Repository>);

impl RepositoryInjection {
    pub fn new(repo: impl Repository) -> Self {
        Self(Box::new(repo))
    }
}

impl Deref for RepositoryInjection {
    type Target = dyn Repository;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

pub struct MemoryRepository {
    users: Vec<User>
}

impl Default for MemoryRepository {
    fn default() -> Self {
        Self {
            users: vec![User::new("Ronaldo".to_string(), (1983, 08, 30))]
        }
    }
}

impl Repository for MemoryRepository {
    fn get_user(&self, user_id: &uuid::Uuid) -> Result<User, Error> {
        self.users
            .iter()
            .find(|u| &u.id == user_id)
            .cloned()
            .ok_or_else(|| Error::new("User not found".to_string(), 404))
    }
}

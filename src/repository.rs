use std::future::{Ready, ready};
use std::ops::Deref;
use std::sync::Arc;
use actix_web::{FromRequest, HttpRequest};
use actix_web::dev::Payload;
use crate::Error;
use crate::user::User;

pub trait Repository: Send + Sync + 'static {
    fn get_user(&self, user_id: &uuid::Uuid) -> Result<User, Error>;
}

pub struct RepositoryInjection(Arc<Box<dyn Repository>>);

impl RepositoryInjection {
    pub fn new(repo: impl Repository) -> Self {
        Self(Arc::new(Box::new(repo)))
    }
}

impl Clone for RepositoryInjection {
    fn clone(&self) -> Self {
        let repo = self.0.clone();
        Self(repo)
    }
}

impl Deref for RepositoryInjection {
    type Target = dyn Repository;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().as_ref()
    }
}

impl FromRequest for RepositoryInjection {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;
    type Config = ();

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        if let Some(injector) = req.app_data::<Self>() {
            let owned_injector = injector.to_owned();
            ready(Ok(owned_injector))
        } else {
            ready(Err(actix_web::error::ErrorBadRequest("No repository injector was found in the request")))
        }
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

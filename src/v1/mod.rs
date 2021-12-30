mod users;

use actix_web::web::{self, ServiceConfig};
use crate::repository::Repository;

pub fn service<R: Repository>(cfg: &mut ServiceConfig) {
    cfg.service(web::scope("/v1").configure(users::service::<R>));
}



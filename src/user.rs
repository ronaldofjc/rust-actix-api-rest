use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: uuid::Uuid,
    pub email: String,
    pub name: String,
    pub birth_date: NaiveDate,
    pub custom_data: CustomData,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "custom_data")]
pub struct CustomData {
    pub random: i32,
}

#[allow(dead_code)]
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

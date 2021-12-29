use chrono::{DateTime, NaiveDate, Utc};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: uuid::Uuid,
    pub name: String,
    pub birth_date: NaiveDate,
    pub custom_data: CustomData,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>
}

impl User {
    pub fn new(name: String, birth_date_ymd: (i32, u32, u32)) -> Self {
        let (year, month, date) = birth_date_ymd;
        let id = uuid::Uuid::parse_str("71802ecd-4eb3-4381-af7e-f737e3a35d5c").unwrap();
        Self {
            id,
            name,
            birth_date: NaiveDate::from_ymd(year, month, date),
            custom_data: CustomData { random: 1 },
            created_at: Some(Utc::now()),
            updated_at: None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomData {
    pub random: u32,
}
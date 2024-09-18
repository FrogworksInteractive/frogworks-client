use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Purchase {
    pub id: i32,
    pub application_id: i32,
    pub iap_id: i32,
    pub user_id: i32,
    pub r#type: String,
    pub source: String,
    pub price: f32,
    pub key: String,
    pub date: String
}
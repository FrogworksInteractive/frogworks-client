use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Sale {
    pub id: i32,
    pub application_id: i32,
    pub title: String,
    pub description: String,
    pub price: f32,
    pub start_date: String,
    pub end_date: String
}
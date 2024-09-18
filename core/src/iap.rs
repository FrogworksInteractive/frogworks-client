use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct IAP {
    pub id: i32,
    pub application_id: i32,
    pub title: String,
    pub description: String,
    pub price: f32,
    pub data: Value
}
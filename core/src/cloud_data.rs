use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct CloudData {
    pub id: i32,
    pub user_id: i32,
    pub application_id: i32,
    pub data: Value,
    pub date: String
}
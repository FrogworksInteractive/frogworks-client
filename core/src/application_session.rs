use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ApplicationSession {
    pub id: i32,
    pub user_id: i32,
    pub application_id: i32,
    pub date: String,
    pub length: i32
}
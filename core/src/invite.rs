use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct Invite {
    pub id: i32,
    pub user_id: i32,
    pub from_user_id: i32,
    pub application_id: i32,
    pub details: Value,
    pub date: String
}
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Session {
    pub id: i32,
    pub identifier: String,
    pub user_id: i32,
    pub hostname: String,
    pub mac_address: String,
    pub platform: String,
    pub start_date: String,
    pub last_activity: String
}
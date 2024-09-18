use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Friend {
    pub id: i32,
    pub user_id: i32,
    pub other_user_id: i32,
    pub date: String
}
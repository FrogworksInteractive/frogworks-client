use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct FriendRequest {
    pub id: i32,
    pub user_id: i32,
    pub from_user_id: i32,
    pub date: String
}
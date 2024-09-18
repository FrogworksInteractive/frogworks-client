use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ApplicationKey {
    pub id: i32,
    pub application_id: i32,
    pub key: String,
    pub r#type: String,
    pub redeemed: bool,
    pub user_id: i32
}
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction {
    pub id: i32,
    pub user_id: i32,
    pub transaction_id: i32,
    pub r#type: String,
    pub date: String
}
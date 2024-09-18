use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Deposit {
    pub id: i32,
    pub user_id: i32,
    pub amount: f32,
    pub source: String,
    pub date: String
}
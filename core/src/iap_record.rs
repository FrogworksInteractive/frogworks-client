use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct IAPRecord {
    pub id: i32,
    pub iap_id: i32,
    pub user_id: i32,
    pub application_id: i32,
    pub date: String,
    pub acknowledged: bool
}
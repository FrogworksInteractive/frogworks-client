use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ApplicationVersion {
    pub id: i32,
    pub application_id: i32,
    pub r#name: String,
    pub platform: String,
    pub release_date: String,
    pub filename: String,
    pub executable: String
}
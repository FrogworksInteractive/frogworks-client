use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Photo {
    pub id: i32,
    pub filename: String,
    pub subfolder: String,
    pub created_at: String
}
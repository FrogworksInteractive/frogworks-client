use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct Activity {
    pub application_id: i32,
    pub description: String,
    pub details: Value
}
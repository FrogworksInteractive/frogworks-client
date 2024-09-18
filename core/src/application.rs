use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Application {
    pub id: i32,
    pub name: String,
    pub package_name: String,
    pub r#type: String,
    pub description: String,
    pub release_date: String,
    pub early_access: bool,
    pub latest_version: String,
    pub supported_platforms: Vec<String>,
    pub genres: Vec<String>,
    pub tags: Vec<String>,
    pub base_price: f32,
    pub owners: Vec<i32>
}
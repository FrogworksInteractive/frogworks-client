use serde::{Deserialize, Serialize};
use crate::activity::Activity;

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub id: i32,
    pub identifier: String,
    pub username: String,
    pub name: String,
    pub email_address: Option<String>,
    pub password: Option<String>,
    pub joined: String,
    pub balance: f32,
    pub profile_photo_id: i32,
    pub activity: Activity,
    pub developer: bool,
    pub administrator: bool,
    pub verified: bool
}
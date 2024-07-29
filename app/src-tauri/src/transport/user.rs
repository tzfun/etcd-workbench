use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SerializableUser {
    pub user: String,
    pub roles: Vec<String>
}
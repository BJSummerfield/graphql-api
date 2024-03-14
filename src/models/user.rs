use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub upn: String,
    // Add other relevant user fields
}

impl User {
    pub fn new(name: String) -> Self {
        User {
            id: "123".to_string(),
            upn: name,
        }
    }
}

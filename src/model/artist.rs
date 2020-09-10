use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Artist {
    pub id: Option<u32>,
    pub name: Option<String>
}

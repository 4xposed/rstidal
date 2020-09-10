pub mod artist;
pub mod album;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ModelType {
    Artist,
    Main,
    Album
}

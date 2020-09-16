pub mod artist;
pub mod album;
pub mod playlist;
pub mod track;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ModelType {
    Album,
    Artist,
    Editorial,
    Main,
    User
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum AudioMode {
    Stereo
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum AudioQuality {
    Lossless,
    #[serde(rename = "HI_RES")]
    HiRes
}

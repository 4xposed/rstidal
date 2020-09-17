pub mod album;
pub mod artist;
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
    User,
    Podcast,
    Contributor,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum AudioMode {
    Mono,
    Stereo,
    #[serde(rename = "SONY_360RA")]
    Sony360RealityAudio,
    #[serde(rename = "DOLBY_ATMOS")]
    DolbyAtmos,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum AudioQuality {
    Lossless,
    #[serde(rename = "HI_RES")]
    Master,
    High,
    Low,
}

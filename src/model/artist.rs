// Use 3rd party
use serde::{Deserialize, Serialize};

// Use local
use crate::model::ModelType;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ArtistType {
    Artist,
    Contributor,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Artist {
    pub id: Option<u32>,
    pub name: Option<String>,
    #[serde(rename(deserialize = "artist_types"))]
    pub artist_types: Option<Vec<ArtistType>>,
    pub url: Option<String>,
    pub picture: Option<String>,
    pub popularity: Option<u16>,
    #[serde(rename = "type")]
    pub _type: Option<ModelType>,
}

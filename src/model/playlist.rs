// Use 3rd party
use serde::{Deserialize, Serialize};

// Use local
use crate::model::artist::Artist;
use crate::model::ModelType;

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Playlist {
  pub uuid: Option<String>,
  pub title: Option<String>,
  pub number_of_tracks: Option<u8>,
  pub number_of_videos: Option<u8>,
  pub description: Option<String>,
  pub duration: Option<u16>,
  pub last_updated: Option<String>,
  pub created: Option<String>,
  pub _type: Option<ModelType>,
  pub public_playlist: Option<bool>,
  pub url: Option<String>,
  pub image: Option<String>,
  pub popularity: Option<u8>,
  pub square_image: Option<String>,
  pub promoted_artists: Option<Vec<Artist>>,
  pub last_item_added_at: Option<String>
}

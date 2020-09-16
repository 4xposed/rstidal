// Use 3rd party
use serde::{Deserialize, Serialize};

// Use local
use crate::model::artist::Artist;
use crate::model::{ModelType, AudioMode, AudioQuality};

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Album {
    pub id: Option<u32>,
    pub title: Option<String>,
    pub duration: Option<u16>,
    pub stream_ready: Option<bool>,
    pub stream_start_date: Option<String>,
    pub allow_streaming: Option<bool>,
    pub premium_streaming_only: Option<bool>,
    pub number_of_tracks: Option<u8>,
    pub number_of_videos: Option<u8>,
    pub number_of_volumes: Option<u8>,
    pub release_date: Option<String>,
    pub copyright: Option<String>,
    pub version: Option<String>,
    pub url: Option<String>,
    pub cover: Option<String>,
    pub video_cover: Option<String>,
    pub explicit: Option<bool>,
    pub upc: Option<String>,
    pub popularity: Option<u8>,
    pub audio_quality: Option<AudioQuality>,
    pub audio_modes: Option<Vec<AudioMode>>,
    pub artists: Option<Vec<Artist>>,
    #[serde(rename = "type")]
    pub _type: Option<ModelType>,
}

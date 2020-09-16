// Use 3rd party
use serde::{Deserialize, Serialize};

use crate::model::artist::Artist;
use crate::model::album::Album;
use crate::model::{AudioMode, AudioQuality};

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Track {
    pub id: Option<u32>,
    pub title: Option<String>,
    pub duration: Option<u16>,
    pub replay_gain: Option<f32>,
    pub peak: Option<f32>,
    pub allow_streaming: Option<bool>,
    pub stream_ready: Option<bool>,
    pub stream_start_date: Option<String>,
    pub premium_streaming_only: Option<bool>,
    pub track_number: Option<u8>,
    pub volume_number: Option<u8>,
    pub version: Option<String>,
    pub popularity: Option<u8>,
    pub copyright: Option<String>,
    pub url: Option<String>,
    pub isrc: Option<String>,
    pub editable: Option<bool>,
    pub explicit: Option<bool>,
    pub audio_quality: Option<AudioQuality>,
    pub audio_modes: Vec<Option<AudioMode>>,
    pub artist: Option<Artist>,
    pub artists: Vec<Option<Artist>>,
    pub album: Option<Album>
}

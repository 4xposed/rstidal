//! Endpoint functions related to playlists

use std::collections::HashMap;

use crate::client::{Tidal, TidalItems};
use crate::error::TidalResult;
use crate::model::playlist::Playlist;
use crate::model::track::Track;

pub struct Playlists<'a>(pub &'a Tidal);

impl Playlists<'_> {
    pub async fn get(&self, id: &str) -> TidalResult<Playlist> {
        let url = format!("/playlists/{}", id);
        let params = HashMap::new();
        let result = self.0.get(&url, &params).await?;
        Tidal::convert_result::<Playlist>(&result)
    }

    pub async fn search(&self, term: &str, limit: Option<u16>) -> TidalResult<Vec<Playlist>> {
        let playlists = self.0.searches().find(term, limit).await?.playlists.items;
        Ok(playlists)
    }

    pub async fn tracks(&self, id: &str) -> TidalResult<Vec<Track>> {
        let url = format!("/playlists/{}/tracks", id);
        let params = HashMap::new();
        let result = self.0.get(&url, &params).await?;
        let tracks = Tidal::convert_result::<TidalItems<Track>>(&result)?.items;
        Ok(tracks)
    }

    pub async fn create(&self, title: &str, description: &str) -> TidalResult<Playlist> {
        let user_id = self.0.user_id()?;
        let url = format!("/users/{}/playlists", user_id);
        let mut form: HashMap<&str, &str> = HashMap::with_capacity(2);
        form.insert("title", title);
        form.insert("description", description);
        let result = self.0.post(&url, &form, None).await?;
        Tidal::convert_result::<Playlist>(&result)
    }

    pub async fn add_tracks(
        &self,
        id: &str,
        tracks: Vec<Track>,
        add_dupes: bool,
    ) -> TidalResult<Playlist> {
        let url = format!("/playlists/{}/items", id);

        // Get etag for the Playlist to be allowed to update the Playlist
        let etag: String = self.0.etag(&url).await?;

        // Convert the list of Tracks to a String with comma separated Track IDs
        let track_ids: Result<Vec<String>, _> = tracks
            .iter()
            .map(|track| {
                track
                    .id
                    .ok_or_else(|| {
                        crate::error::TidalError::Config("Track ID is required".to_string())
                    })
                    .map(|id| id.to_string())
            })
            .collect();
        let track_ids: String = track_ids?.join(",");

        let on_dupes: String = if add_dupes {
            "ADD".to_owned()
        } else {
            "FAIL".to_owned()
        };

        let mut form: HashMap<&str, &str> = HashMap::with_capacity(2);
        form.insert("trackIds", &track_ids);
        form.insert("onDupes", &on_dupes);

        // Submit request to add the Tracks to the Playlist
        self.0.post(&url, &form, Some(etag)).await?;

        // Get updated Playlist
        self.get(id).await
    }

    pub async fn user_playlists(&self) -> TidalResult<Vec<Playlist>> {
        let user_id = self.0.user_id()?;
        let url = format!("/users/{}/playlists", user_id);
        let params = HashMap::new();
        let result = self.0.get(&url, &params).await?;
        let playlists = Tidal::convert_result::<TidalItems<Playlist>>(&result)?.items;
        Ok(playlists)
    }
}

#[cfg(test)]
mod tests {

    // Tests have been moved to separate test files for better organization
}

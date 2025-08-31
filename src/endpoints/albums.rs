//! Endpoint functions relating to albums

use std::collections::HashMap;

use crate::client::{Tidal, TidalItems};
use crate::error::TidalResult;
use crate::model::album::Album;
use crate::model::track::Track;

pub struct Albums<'a>(pub &'a Tidal);

impl Albums<'_> {
    pub async fn get(&self, id: &str) -> TidalResult<Album> {
        let url = format!("/albums/{}", id);
        let params = HashMap::new();
        let result = self.0.get(&url, &params).await?;
        Tidal::convert_result::<Album>(&result)
    }

    pub async fn search(&self, term: &str, limit: Option<u16>) -> TidalResult<Vec<Album>> {
        let albums = self.0.searches().find(term, limit).await?.albums.items;
        Ok(albums)
    }

    pub async fn tracks(&self, id: &str) -> TidalResult<Vec<Track>> {
        let url = format!("/albums/{}/tracks", id);
        let params = HashMap::new();
        let result = self.0.get(&url, &params).await?;
        let tracks = Tidal::convert_result::<TidalItems<Track>>(&result)?.items;
        Ok(tracks)
    }
}

#[cfg(test)]
mod tests {
    // Tests have been moved to separate test files for better organization
}

//! Endpoint functions relateed to artists

use std::collections::HashMap;

use crate::client::{Tidal, TidalItems};
use crate::error::TidalResult;
use crate::model::album::Album;
use crate::model::artist::Artist;

pub struct Artists<'a>(pub &'a Tidal);

impl Artists<'_> {
    pub async fn get(&self, id: &str) -> TidalResult<Artist> {
        let url = format!("/artists/{}", id);
        let params = HashMap::new();
        let result = self.0.get(&url, &params).await?;
        Tidal::convert_result::<Artist>(&result)
    }

    pub async fn search(&self, term: &str, limit: Option<u16>) -> TidalResult<Vec<Artist>> {
        let artists = self.0.searches().find(term, limit).await?.artists.items;
        Ok(artists)
    }

    pub async fn albums(&self, id: &str) -> TidalResult<Vec<Album>> {
        let url = format!("/artists/{}/albums", id);
        let params = HashMap::new();
        let result = self.0.get(&url, &params).await?;
        let albums = Tidal::convert_result::<TidalItems<Album>>(&result)?.items;
        Ok(albums)
    }
}

#[cfg(test)]
mod tests {
    // Tests have been moved to separate test files for better organization
}

//! Endpoint functions related to tracks

use crate::client::Tidal;
use crate::error::TidalResult;
use crate::model::track::Track;

pub struct Tracks<'a>(pub &'a Tidal);

impl Tracks<'_> {
    pub async fn search(&self, term: &str, limit: Option<u16>) -> TidalResult<Vec<Track>> {
        let tracks = self.0.searches().find(term, limit).await?.tracks.items;
        Ok(tracks)
    }
}

#[cfg(test)]
mod tests {

    // Tests have been moved to separate test files for better organization
}

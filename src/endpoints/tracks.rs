//! Endpoint functions related to playlists

use crate::client::*;
use crate::model::track::*;

pub struct Tracks<'a>(pub &'a Tidal);

impl Tracks<'_> {
    pub async fn search(&self, term: &str, limit: Option<u16>) -> ClientResult<Vec<Track>> {
        let tracks = self.0.search(term, limit).await?.tracks.items;
        Ok(tracks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::tests::{client, mock_request_success_from_file};
    use mockito::Matcher;

    #[tokio::test]
    async fn search() {
        let _mock = mock_request_success_from_file(
            "GET",
            "/search",
            vec![
                Matcher::UrlEncoded("countryCode".into(), "US".into()),
                Matcher::UrlEncoded("query".into(), "trivium".into()),
            ],
            "tests/files/search.json",
        )
        .create();

        let result: Vec<Track> = client().tracks().search("trivium", None).await.unwrap();

        assert_eq!(result.len(), 10);
    }
}

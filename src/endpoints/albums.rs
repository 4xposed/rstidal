//! Endpoint functions relating to albums

use std::collections::HashMap;

use crate::client::{ClientResult, Tidal, TidalItems};
use crate::model::album::Album;
use crate::model::track::Track;

pub struct Albums<'a>(pub &'a Tidal);

impl Albums<'_> {
    pub async fn get(self, id: &str) -> ClientResult<Album> {
        let url = format!("/albums/{}", id);
        let result = self.0.get(&url, &mut HashMap::new()).await?;
        Tidal::convert_result::<Album>(&result)
    }

    pub async fn search(&self, term: &str, limit: Option<u16>) -> ClientResult<Vec<Album>> {
        let albums = self.0.search(term, limit).await?.albums.items;
        Ok(albums)
    }

    pub async fn tracks(&self, id: &str) -> ClientResult<Vec<Track>> {
        let url = format!("/albums/{}/tracks", id);
        let result = self.0.get(&url, &mut HashMap::new()).await?;
        let tracks = Tidal::convert_result::<TidalItems<Track>>(&result)?.items;
        Ok(tracks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::tests::{client, mock_request_success_from_file};
    use mockito::Matcher;

    #[tokio::test]
    async fn get() {
        let _mock = mock_request_success_from_file(
            "GET",
            "/albums/79914998",
            vec![Matcher::UrlEncoded("countryCode".into(), "US".into())],
            "tests/files/album.json",
        );

        let result: Album = client().album("79914998").await.unwrap();
        let expected_result = Album {
            id: Some(79914998),
            title: Some("My Album".to_owned()),
            ..Default::default()
        };
        assert_eq!(result.id, expected_result.id);
        assert_eq!(result.title, expected_result.title);
    }

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

        let result: Vec<Album> = client().search_album("trivium", None).await.unwrap();

        assert_eq!(result.len(), 10);
    }

    #[tokio::test]
    async fn tracks() {
        let _mock = mock_request_success_from_file(
            "GET",
            "/albums/79914998/tracks",
            vec![Matcher::UrlEncoded("countryCode".into(), "US".into())],
            "tests/files/album_tracks.json",
        );

        let result: Vec<Track> = client().album_tracks("79914998").await.unwrap();
        let expected_first_result = Track {
            title: Some("The Sin and the Sentence".to_owned()),
            ..Default::default()
        };
        assert_eq!(result[0].title, expected_first_result.title);
    }
}

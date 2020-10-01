//! Endpoint functions relateed to artists

use std::collections::HashMap;

use crate::client::{ClientResult, Tidal, TidalItems};
use crate::model::album::Album;
use crate::model::artist::Artist;

pub struct Artists<'a>(pub &'a Tidal);

impl Artists<'_> {
    pub async fn get(&self, id: &str) -> ClientResult<Artist> {
        let url = format!("/artists/{}", id);
        let result = self.0.get(&url, &mut HashMap::new()).await?;
        Tidal::convert_result::<Artist>(&result)
    }

    pub async fn search(&self, term: &str, limit: Option<u16>) -> ClientResult<Vec<Artist>> {
        let artists = self.0.search(term, limit).await?.artists.items;
        Ok(artists)
    }

    pub async fn albums(&self, id: &str) -> ClientResult<Vec<Album>> {
        let url = format!("/artists/{}/albums", id);
        let result = self.0.get(&url, &mut HashMap::new()).await?;
        let albums = Tidal::convert_result::<TidalItems<Album>>(&result)?.items;
        Ok(albums)
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
            "/artists/37312",
            vec![Matcher::UrlEncoded("countryCode".into(), "US".into())],
            "tests/files/artist.json",
        );

        let result: Artist = client().artists().get("37312").await.unwrap();
        let expected_result = Artist {
            id: Some(37312),
            name: Some("myband".to_owned()),
            ..Default::default()
        };
        assert_eq!(result.id, expected_result.id);
        assert_eq!(result.name, expected_result.name);
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

        let result: Vec<Artist> = client().artists().search("trivium", None).await.unwrap();

        assert_eq!(result.len(), 10);
    }

    #[tokio::test]
    async fn albums() {
        let _mock = mock_request_success_from_file(
            "GET",
            "/artists/37312/albums",
            vec![Matcher::UrlEncoded("countryCode".into(), "US".into())],
            "tests/files/artist_albums.json",
        );

        let result: Vec<Album> = client().artists().albums("37312").await.unwrap();
        let expected_first_result = Album {
            id: Some(138458220),
            title: Some("What The Dead Men Say".to_owned()),
            ..Default::default()
        };
        assert_eq!(result[0].id, expected_first_result.id);
        assert_eq!(result[0].title, expected_first_result.title);
    }
}

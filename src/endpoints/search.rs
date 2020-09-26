//! Endpoint functions related to search

use std::collections::HashMap;

use crate::client::*;

pub struct Search<'a>(pub &'a Tidal);

impl Search<'_> {
    pub async fn find(&self, term: &str, limit: Option<u16>) -> ClientResult<TidalSearch> {
        let url = "/search";
        let limit = if let Some(limit) = limit { limit } else { 10 };
        let mut params: HashMap<String, String> = HashMap::new();
        params.insert("query".to_owned(), term.to_owned());
        params.insert("limit".to_owned(), limit.to_string());
        let result = self.0.get(&url, &mut params).await?;
        Tidal::convert_result::<TidalSearch>(&result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::tests::{client, mock_request_success_from_file};
    use mockito::Matcher;

    #[tokio::test]
    async fn find() {
        let _mock = mock_request_success_from_file(
            "GET",
            "/search",
            vec![
                Matcher::UrlEncoded("countryCode".into(), "US".into()),
                Matcher::UrlEncoded("query".into(), "trivium".into()),
                Matcher::UrlEncoded("limit".into(), "10".into()),
            ],
            "tests/files/search.json",
        )
        .create();

        let result: TidalSearch = client().searches().find("trivium", None).await.unwrap();

        assert_eq!(result.artists.items.len(), 10);
        assert_eq!(result.albums.items.len(), 10);
        assert_eq!(result.tracks.items.len(), 10);
        assert_eq!(result.playlists.items.len(), 10);
    }
}

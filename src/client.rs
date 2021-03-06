// Use 3rd party
use log::{debug, warn};
use reqwest::header::HeaderMap;
use reqwest::{Client, Method, Response, StatusCode};
use serde::Deserialize;
use thiserror::Error;

#[cfg(test)]
use mockito;

// Use built-in library
use std::borrow::Cow;
use std::collections::HashMap;

// Use internal modules
use crate::auth::{Session, TidalCredentials};
use crate::model::album::Album;
use crate::model::artist::Artist;
use crate::model::playlist::Playlist;
use crate::model::track::Track;

// Possible errors returned from `rstidal` client.
#[derive(Debug, Error)]
pub enum ClientError {
    #[error("request unauthorized")]
    Unauthorized,
    #[error("tidal error: {0}")]
    Api(#[from] ApiError),
    #[error("etag heeader parse error")]
    ParseEtag,
    #[error("json parse error: {0}")]
    ParseJSON(#[from] serde_json::Error),
    #[error("request error: {0}")]
    Request(#[from] reqwest::Error),
    #[error("status code: {0}")]
    StatusCode(StatusCode),
}

impl ClientError {
    async fn from_response(response: Response) -> Self {
        match response.status() {
            StatusCode::UNAUTHORIZED => Self::Unauthorized,
            status @ StatusCode::FORBIDDEN | status @ StatusCode::NOT_FOUND => response
                .json::<ApiError>()
                .await
                .map_or_else(|_| status.into(), Into::into),
            status => status.into(),
        }
    }
}

impl From<StatusCode> for ClientError {
    fn from(code: StatusCode) -> Self {
        Self::StatusCode(code)
    }
}
#[derive(Debug, Error, Deserialize)]
pub enum ApiError {
    #[error("{status}: {message}")]
    Regular {
        status: u16,
        #[serde(alias = "userMessage")]
        message: String,
    },
}

pub type ClientResult<T> = Result<T, ClientError>;

#[derive(Default, Debug, Deserialize)]
pub struct TidalItems<T> {
    pub items: Vec<T>,
}

#[derive(Default, Debug, Deserialize)]
pub struct TidalSearch {
    pub artists: TidalItems<Artist>,
    pub albums: TidalItems<Album>,
    pub playlists: TidalItems<Playlist>,
    pub tracks: TidalItems<Track>,
}

// Tidal API

pub struct Tidal {
    client: Client,
    pub(crate) credentials: TidalCredentials,
}

impl Tidal {
    #[must_use]
    pub fn new(credentials: TidalCredentials) -> Self {
        if credentials.session.is_none() {
            panic!("A session needs to be obtatined before using Tidal");
        }

        Self {
            client: Client::new(),
            credentials,
        }
    }

    pub fn user_id(&self) -> u32 {
        // Here it's safe to use unwrap because in ::new() we already checked that there's a valid
        // session
        self.credentials.session.as_ref().unwrap().user_id
    }

    async fn api_call(
        &self,
        method: Method,
        url: &str,
        query: Option<&HashMap<String, String>>,
        payload: Option<&HashMap<&str, &str>>,
        etag: Option<String>,
    ) -> ClientResult<Response> {
        #[cfg(not(test))]
        let base_url: &str = "https://api.tidalhifi.com/v1";
        #[cfg(test)]
        let base_url: &str = &mockito::server_url();

        let mut url: Cow<str> = url.into();
        if !url.starts_with("http") {
            url = [base_url, &url].concat().into();
        }

        let Session { session_id, country_code, .. } = self.credentials.session.as_ref().unwrap();

        let mut headers = HeaderMap::new();
        headers.insert("X-Tidal-SessionId", session_id.parse().unwrap());
        headers.insert("Origin", "http://listen.tidal.com".parse().unwrap());
        if let Some(etag) = etag {
            headers.insert("If-None-Match", etag.parse().unwrap());
        }

        // Tidal's API requires countryCode to always be passed
        let mut query_params: HashMap<String, String> = HashMap::new();
        query_params.insert("countryCode".to_owned(), country_code.to_owned());

        if let Some(query) = query {
            for (key, value) in query.iter() {
                query_params.insert(key.clone(), value.clone());
            }
        }

        let response = {
            let builder = self
                .client
                .request(method, &url.into_owned())
                .headers(headers)
                .query(&query_params);

            // Only add payload when sent
            let builder = if let Some(form) = payload {
                builder.form(form)
            } else {
                builder
            };

            debug!("request builder: {:?}", builder);
            builder.send().await.map_err(ClientError::from)?
        };

        debug!("response content: {:?}", response);
        if response.status().is_success() {
            Ok(response)
        } else {
            Err(ClientError::from_response(response).await)
        }
    }

    pub async fn etag(&self, url: &str) -> ClientResult<String> {
        // Tidal's API requires countryCode to always be passed
        let headers = self
            .api_call(Method::GET, &url, None, None, None)
            .await?
            .headers()
            .clone();

        if let Ok(etag) = headers
            .get("etag")
            .expect("etag header to be present")
            .to_str()
        {
            Ok(etag.to_owned())
        } else {
            Err(ClientError::ParseEtag)
        }
    }

    pub async fn get(
        &self,
        url: &str,
        params: &mut HashMap<String, String>,
    ) -> ClientResult<String> {
        self.api_call(Method::GET, &url, Some(params), None, None)
            .await?
            .text()
            .await
            .map_err(Into::into)
    }

    pub async fn post(
        &self,
        url: &str,
        payload: &HashMap<&str, &str>,
        etag: Option<String>,
    ) -> ClientResult<String> {
        self.api_call(Method::POST, &url, None, Some(payload), etag)
            .await?
            .text()
            .await
            .map_err(Into::into)
    }

    pub async fn put(
        &self,
        url: &str,
        payload: &HashMap<&str, &str>,
        etag: String,
    ) -> ClientResult<String> {
        self.api_call(Method::PUT, url, None, Some(payload), Some(etag))
            .await?
            .text()
            .await
            .map_err(Into::into)
    }

    // The following functions are for backward compatibility only
    //
    pub async fn search(&self, term: &str, limit: Option<u16>) -> ClientResult<TidalSearch> {
        warn!("DEPRECATION WARNING!: This method will be deprecated in the next version. Please favor using .searches().find()");
        self.searches().find(term, limit).await
    }

    pub async fn artist(&self, id: &str) -> ClientResult<Artist> {
        warn!("DEPRECATION WARNING!: This method will be deprecated in the next version. Please favor using .artists().get()");
        self.artists().get(id).await
    }

    pub async fn search_artist(&self, term: &str, limit: Option<u16>) -> ClientResult<Vec<Artist>> {
        warn!("DEPRECATION WARNING!: This method will be deprecated in the next version. Please favor using .artists().search()");
        self.artists().search(term, limit).await
    }

    pub async fn album(&self, id: &str) -> ClientResult<Album> {
        warn!("DEPRECATION WARNING!: This method will be deprecated in the next version. Please favor using .albums().get()");
        self.albums().get(id).await
    }

    pub async fn artist_albums(&self, id: &str) -> ClientResult<Vec<Album>> {
        warn!("DEPRECATION WARNING!: This method will be deprecated in the next version. Please favor using .artists().albums()");
        self.artists().albums(id).await
    }

    pub async fn search_album(&self, term: &str, limit: Option<u16>) -> ClientResult<Vec<Album>> {
        warn!("DEPRECATION WARNING!: This method will be deprecated in the next version. Please favor using .albums().search()");
        self.albums().search(term, limit).await
    }

    pub async fn album_tracks(&self, id: &str) -> ClientResult<Vec<Track>> {
        warn!("DEPRECATION WARNING!: This method will be deprecated in the next version. Please favor using .albums().tracks()");
        self.albums().tracks(id).await
    }

    pub async fn search_track(&self, term: &str, limit: Option<u16>) -> ClientResult<Vec<Track>> {
        warn!("DEPRECATION WARNING!: This method will be deprecated in the next version. Please favor using .tracks().search()");
        self.tracks().search(term, limit).await
    }

    pub async fn playlist(&self, id: &str) -> ClientResult<Playlist> {
        warn!("DEPRECATION WARNING!: This method will be deprecated in the next version. Please favor using .playlists().get()");
        self.playlists().get(id).await
    }

    pub async fn search_playlist(&self, term: &str, limit: Option<u16>) -> ClientResult<Vec<Playlist>> {
        warn!("DEPRECATION WARNING!: This method will be deprecated in the next version. Please favor using .playlists().search()");
        self.playlists().search(term, limit).await
    }

    pub async fn user_playlists(&self) -> ClientResult<Vec<Playlist>> {
        warn!("DEPRECATION WARNING!: This method will be deprecated in the next version. Please favor using .playlists().user_playlists()");
        self.playlists().user_playlists().await
    }

    pub async fn playlist_tracks(&self, id: &str) -> ClientResult<Vec<Track>> {
        warn!("DEPRECATION WARNING!: This method will be deprecated in the next version. Please favor using .playlists().tracks()");
        self.playlists().tracks(id).await
    }

    pub async fn playlist_add_tracks(&self, id: &str, tracks: Vec<Track>, add_dupes: bool) -> ClientResult<Playlist> {
        warn!("DEPRECATION WARNING!: This method will be deprecated in the next version. Please favor using .playlists().add_tracks()");
        self.playlists().add_tracks(id, tracks, add_dupes).await
    }

    pub async fn create_playlist(&self, title: &str, description: &str) -> ClientResult<Playlist> {
        warn!("DEPRECATION WARNING!: This method will be deprecated in the next version. Please favor using .playlists().create()");
        self.playlists().create(title, description).await
    }

    pub fn convert_result<'a, T: Deserialize<'a>>(input: &'a str) -> ClientResult<T> {
        serde_json::from_str::<T>(input).map_err(Into::into)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::auth::Session;
    use mockito::{mock, Matcher};

    #[tokio::test]
    async fn client_get() {
        let mut params: HashMap<String, String> = HashMap::new();

        // All requesets going to Tidal ned to append ?countryCode=$USER_REGION
        let _mock = mock_request_success(
            "GET",
            "/",
            vec![Matcher::UrlEncoded("countryCode".into(), "US".into())],
            r#"{"result": "ok"}"#,
        );

        let client = client();
        let response = client.get("/", &mut params).await.unwrap();
        assert_eq!(response, r#"{"result": "ok"}"#)
    }

    #[tokio::test]
    async fn client_search() {
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

        let result: TidalSearch = client().search("trivium", None).await.unwrap();

        assert_eq!(result.artists.items.len(), 10);
        assert_eq!(result.albums.items.len(), 10);
        assert_eq!(result.tracks.items.len(), 10);
        assert_eq!(result.playlists.items.len(), 10);
    }

    #[tokio::test]
    async fn client_artist() {
        let _mock = mock_request_success_from_file(
            "GET",
            "/artists/37312",
            vec![Matcher::UrlEncoded("countryCode".into(), "US".into())],
            "tests/files/artist.json",
        )
        .create();

        let result: Artist = client().artist("37312").await.unwrap();
        let expected_result = Artist {
            id: Some(37312),
            name: Some("myband".to_owned()),
            ..Default::default()
        };
        assert_eq!(result.id, expected_result.id);
        assert_eq!(result.name, expected_result.name);
    }

    #[tokio::test]
    async fn client_search_artist() {
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

        let result: Vec<Artist> = client().search_artist("trivium", None).await.unwrap();

        assert_eq!(result.len(), 10);
    }

    #[tokio::test]
    async fn client_artist_albums() {
        let _mock = mock_request_success_from_file(
            "GET",
            "/artists/37312/albums",
            vec![Matcher::UrlEncoded("countryCode".into(), "US".into())],
            "tests/files/artist_albums.json",
        );

        let result: Vec<Album> = client().artist_albums("37312").await.unwrap();
        let expected_first_result = Album {
            id: Some(138458220),
            title: Some("What The Dead Men Say".to_owned()),
            ..Default::default()
        };
        assert_eq!(result[0].id, expected_first_result.id);
        assert_eq!(result[0].title, expected_first_result.title);
    }

    #[tokio::test]
    async fn client_album() {
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
    async fn client_search_album() {
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
    async fn client_album_tracks() {
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

    #[tokio::test]
    async fn client_search_tracks() {
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

        let result: Vec<Track> = client().search_track("trivium", None).await.unwrap();

        assert_eq!(result.len(), 10);
    }

    #[tokio::test]
    async fn client_playlist() {
        let _mock = mock_request_success_from_file(
            "GET",
            "/playlists/7ce7df87-6d37-4465-80db-84535a4e44a4",
            vec![Matcher::UrlEncoded("countryCode".into(), "US".into())],
            "tests/files/playlist.json",
        );

        let result: Playlist = client()
            .playlist("7ce7df87-6d37-4465-80db-84535a4e44a4")
            .await
            .unwrap();
        let expected_result = Playlist {
            uuid: Some("7ce7df87-6d37-4465-80db-84535a4e44a4".to_owned()),
            title: Some("Metal - TIDAL Masters".to_owned()),
            ..Default::default()
        };
        assert_eq!(result.uuid, expected_result.uuid);
        assert_eq!(result.title, expected_result.title);
    }

    #[tokio::test]
    async fn client_user_playlists() {
        let _mock = mock_request_success_from_file(
            "GET",
            "/users/1234/playlists",
            vec![Matcher::UrlEncoded("countryCode".into(), "US".into())],
            "tests/files/user_playlists.json",
        );

        let result: Vec<Playlist> = client().user_playlists().await.unwrap();
        let expected_result = Playlist {
            uuid: Some("8edf5a89-fec4-4aa3-80ab-9e00a83633a2".to_owned()),
            title: Some("roadtrip".to_owned()),
            ..Default::default()
        };
        assert_eq!(result[0].uuid, expected_result.uuid);
        assert_eq!(result[0].title, expected_result.title);
    }

    #[tokio::test]
    async fn client_playlist_tracks() {
        let _mock = mock_request_success_from_file(
            "GET",
            "/playlists/7ce7df87-6d37-4465-80db-84535a4e44a4/tracks",
            vec![Matcher::UrlEncoded("countryCode".into(), "US".into())],
            "tests/files/playlist_tracks.json",
        );

        let result: Vec<Track> = client()
            .playlist_tracks("7ce7df87-6d37-4465-80db-84535a4e44a4")
            .await
            .unwrap();
        let expected_first_result = Track {
            title: Some("FULL OF HEALTH".to_owned()),
            ..Default::default()
        };
        assert_eq!(result[0].title, expected_first_result.title);
    }

    #[tokio::test]
    async fn client_playlist_add_tracks() {
        let _mock_reload_playlist = mock_request_success_from_file(
            "GET",
            "/playlists/7ce7df87-6d37-4465-80db-84535a4e44a4",
            vec![Matcher::UrlEncoded("countryCode".into(), "US".into())],
            "tests/files/playlist.json",
        );

        let track_1 = Track {
            id: Some(79914998),
            ..Default::default()
        };
        let track_2 = Track {
            id: Some(7915000),
            ..Default::default()
        };
        let tracks = vec![track_1, track_2];

        let _mock_etag_req = mock(
            "GET",
            "/playlists/7ce7df87-6d37-4465-80db-84535a4e44a4/items",
        )
        .match_query(Matcher::UrlEncoded("countryCode".into(), "US".into()))
        .with_body("")
        .with_header("etag", "123457689")
        .create();

        let mock_update_playlist = mock(
            "POST",
            "/playlists/7ce7df87-6d37-4465-80db-84535a4e44a4/items",
        )
        .match_query(Matcher::UrlEncoded("countryCode".into(), "US".into()))
        .match_header("if-none-match", "123457689")
        .with_body(r#"{ "lastUpdated": 1600273268158, "addedItemIds": [ 79914999, 79915000 ] }"#)
        .create();

        let _result: Playlist = client()
            .playlist_add_tracks("7ce7df87-6d37-4465-80db-84535a4e44a4", tracks, false)
            .await
            .unwrap();
        mock_update_playlist.assert();
    }

    fn mock_request_success(
        method: &str,
        path: &str,
        query: Vec<Matcher>,
        body: &str,
    ) -> mockito::Mock {
        mock(method, path)
            .match_query(Matcher::AllOf(query))
            .with_status(200)
            .with_body(body)
            .create()
    }

    pub fn mock_request_success_from_file(
        method: &str,
        path: &str,
        query: Vec<Matcher>,
        file_path: &str,
    ) -> mockito::Mock {
        mock(method, path)
            .match_query(Matcher::AllOf(query))
            .with_status(200)
            .with_body_from_file(file_path)
            .create()
    }

    pub fn client() -> Tidal {
        Tidal::new(credential())
    }

    fn credential() -> TidalCredentials {
        let session: Session = Session {
            user_id: 1234,
            session_id: "session-id-1".to_owned(),
            country_code: "US".to_owned(),
        };
        TidalCredentials {
            token: "some_token".to_owned(),
            session: Some(session),
        }
    }
}

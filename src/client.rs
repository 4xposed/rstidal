// Use 3rd party
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
use crate::model::artist::Artist;
use crate::model::album::Album;
use crate::model::playlist::Playlist;
use crate::model::track::Track;
use crate::auth::TidalCredentials;

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
        message: String
    }
}

type ClientResult<T> = Result<T, ClientError>;

#[derive(Default, Debug, Deserialize)]
pub struct TidalItems<T> {
    pub items: Vec<T>
}

// Tidal API
pub struct Tidal {
    client: Client,
    credentials: TidalCredentials
}

impl Tidal {
    #[must_use]
    pub fn new(credentials: TidalCredentials) -> Self {
        if credentials.to_owned().session_info.unwrap().session_id.is_none() {
            panic!("You need an authenticated credential to use Tidal");
        };

        Self {
            client: Client::new(),
            credentials
        }
    }

    fn session_id(&self) -> String {
        let credentials = self.credentials.clone();
        let session_id = match &credentials.session_info {
            None => panic!("A session needs to be obtained before using Tidal"),
            Some(session_info) => {
                match &session_info.session_id {
                    Some(session_id) => session_id,
                    None => panic!("An active sessions needs to be obtained before using Tidal")
                }
            }
        };
        session_id.to_owned()
    }

    fn country_code(&self) -> String {
        self.credentials.session_info.as_ref().unwrap().country_code.to_owned()
    }

    fn user_id(&self) -> u32 {
        self.credentials.session_info.as_ref().unwrap().user_id.unwrap()
    }

    async fn api_call(&self, method: Method, url: &str, query: Option<&HashMap<String, String>>, payload: Option<&HashMap<&str, &str>>, etag: Option<String>) -> ClientResult<Response> {
        #[cfg(not(test))]
        let base_url: &str = "https://api.tidalhifi.com/v1";
        #[cfg(test)]
        let base_url: &str = &mockito::server_url();

        let mut url: Cow<str> = url.into();
        if !url.starts_with("http") {
            url = [base_url, &url].concat().into();
        }

        let mut headers = HeaderMap::new();
        headers.insert("X-Tidal-SessionId", self.session_id().parse().unwrap());
        headers.insert("Origin", "http://listen.tidal.com".parse().unwrap());
        if let Some(etag) = etag {
            headers.insert("If-None-Match", etag.parse().unwrap());
        }

        let mut query_params: HashMap<String, String> = HashMap::new();
        query_params.insert("countryCode".to_owned(), self.country_code());

        if let Some(query) = query {
            for (key, value) in query.iter() {
                query_params.insert(key.clone(), value.clone());
            };
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

            builder.send().await.map_err(ClientError::from)?
        };

        if response.status().is_success() {
            Ok(response)
        } else {
            Err(ClientError::from_response(response).await)
        }
    }

    pub async fn etag(&self, url: &str) -> ClientResult<String> {
        // Tidal's API requires countryCode to always be passed
        let headers = self.api_call(Method::GET, &url, None, None, None).await?.headers().clone();

        if let Ok(etag) = headers.get("etag").unwrap().to_str() {
            Ok(etag.to_owned())
        } else {
            Err(ClientError::ParseEtag)
        }
    }

    pub async fn get(&self, url: &str, params: &mut HashMap<String, String>) -> ClientResult<String> {
        // Tidal's API requires countryCode to always be passed
        self.api_call(Method::GET, &url, Some(params), None, None).await?.text().await.map_err(Into::into)
    }

    pub async fn post(&self, url: &str, payload: &HashMap<&str, &str>, etag: String) -> ClientResult<String> {
        self.api_call(Method::POST, &url, None, Some(payload), Some(etag)).await?.text().await.map_err(Into::into)
    }

    pub async fn put(&self, url: &str, payload: &HashMap<&str, &str>, etag: String) -> ClientResult<String> {
        self.api_call(Method::PUT, url, None, Some(payload), Some(etag)).await?.text().await.map_err(Into::into)
    }

    //pub async fn delete(&self, url: &str, payload: &Value, etag: String) -> ClientResult<String> {
        //self.api_call(Method::DELETE, url, Some(payload), Some(etag).await
    //}

    pub async fn artist(&self, id: &str) -> ClientResult<Artist> {
        let url = format!("/artists/{}", id);
        let result = self.get(&url, &mut HashMap::new()).await?;
        Self::convert_result::<Artist>(&result)
    }

    pub async fn artist_albums(&self, id: &str) -> ClientResult<Vec<Album>> {
        let url = format!("/artists/{}/albums", id);
        let result = self.get(&url, &mut HashMap::new()).await?;
        let albums = Self::convert_result::<TidalItems<Album>>(&result)?.items;
        Ok(albums)
    }

    pub async fn album(&self, id: &str) -> ClientResult<Album> {
        let url = format!("/albums/{}", id);
        let result = self.get(&url, &mut HashMap::new()).await?;
        Self::convert_result::<Album>(&result)
    }

    pub async fn album_tracks(&self, id: &str) -> ClientResult<Vec<Track>> {
        let url = format!("/albums/{}/tracks", id);
        let result = self.get(&url, &mut HashMap::new()).await?;
        let tracks = Self::convert_result::<TidalItems<Track>>(&result)?.items;
        Ok(tracks)
    }

    pub async fn playlist(&self, id: &str) -> ClientResult<Playlist> {
        let url = format!("/playlists/{}", id);
        let result = self.get(&url, &mut HashMap::new()).await?;
        Self::convert_result::<Playlist>(&result)
    }

    pub async fn user_playlists(&self) -> ClientResult<Vec<Playlist>> {
        let user_id = self.user_id();
        let url = format!("/users/{}/playlists", user_id);
        let result = self.get(&url, &mut HashMap::new()).await?;
        let playlists = Self::convert_result::<TidalItems<Playlist>>(&result)?.items;
        Ok(playlists)
    }

    pub async fn playlist_tracks(&self, id: &str) -> ClientResult<Vec<Track>> {
        let url = format!("/playlists/{}/tracks", id);
        let result = self.get(&url, &mut HashMap::new()).await?;
        let tracks = Self::convert_result::<TidalItems<Track>>(&result)?.items;
        Ok(tracks)
    }

    pub async fn playlist_add_tracks(&self, id: &str, tracks: Vec<Track>, add_dupes: bool) -> ClientResult<Playlist> {
        let url = format!("/playlists/{}/items", id);
        let etag: String = self.etag(&url).await?;
        let track_ids: Vec<String> = tracks.iter().map(|track| track.id.unwrap().to_string()).collect();
        let track_ids: String = track_ids.join(",");

        let on_dupes: String = if add_dupes {
            "ADD".to_owned()
        } else {
            "FAIL".to_owned()
        };

        let mut form: HashMap<&str, &str> = HashMap::new();
        form.insert("trackIds", &track_ids);
        form.insert("onDupes", &on_dupes);

        self.post(&url, &form, etag).await?;
        self.playlist(id).await
    }

    fn convert_result<'a, T: Deserialize<'a>>(input: &'a str) -> ClientResult<T> {
        serde_json::from_str::<T>(input).map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::mock;
    use crate::auth::SessionInfo;

    #[tokio::test]
    async fn client_get() {
        let mut params: HashMap<String, String> = HashMap::new();

        // All requesets going to Tidal ned to append ?countryCode=$USER_REGION
        let _mock = mock_request_success(
            "GET",
            "/?countryCode=US",
            r#"{"result": "ok"}"#
        );

        let client = client();
        let response = client.get("/", &mut params).await.unwrap();
        assert_eq!(response, r#"{"result": "ok"}"#)
    }

    #[tokio::test]
    async fn client_artist() {
        let _mock = mock_request_success_from_file(
            "GET",
            "/artists/37312?countryCode=US",
            "tests/files/artist.json"
        );

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
    async fn client_artist_albums() {
        let _mock = mock_request_success_from_file(
            "GET",
            "/artists/37312/albums?countryCode=US",
            "tests/files/artist_albums.json"
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
            "/albums/79914998?countryCode=US",
            "tests/files/album.json"
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
    async fn client_album_tracks() {
        let _mock = mock_request_success_from_file(
            "GET",
            "/albums/79914998/tracks?countryCode=US",
            "tests/files/album_tracks.json"
        );

        let result: Vec<Track> = client().album_tracks("79914998").await.unwrap();
        let expected_first_result = Track {
            title: Some("The Sin and the Sentence".to_owned()),
            ..Default::default()
        };
        assert_eq!(result[0].title, expected_first_result.title);
    }

    #[tokio::test]
    async fn client_playlist() {
        let _mock = mock_request_success_from_file(
            "GET",
            "/playlists/7ce7df87-6d37-4465-80db-84535a4e44a4?countryCode=US",
            "tests/files/playlist.json"
        );

        let result: Playlist = client().playlist("7ce7df87-6d37-4465-80db-84535a4e44a4").await.unwrap();
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
            "/users/1234/playlists?countryCode=US",
            "tests/files/user_playlists.json"
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
            "/playlists/7ce7df87-6d37-4465-80db-84535a4e44a4/tracks?countryCode=US",
            "tests/files/playlist_tracks.json"
        );

        let result: Vec<Track> = client().playlist_tracks("7ce7df87-6d37-4465-80db-84535a4e44a4").await.unwrap();
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
            "/playlists/7ce7df87-6d37-4465-80db-84535a4e44a4?countryCode=US",
            "tests/files/playlist.json"
        );

        let track_1 = Track {
            id: Some(79914998),
            ..Default::default()
        };
        let track_2 = Track {
            id: Some(7915000),
            ..Default::default()
        };
        let tracks = vec!(track_1, track_2);

        let _mock_etag_req = mock("GET", "/playlists/7ce7df87-6d37-4465-80db-84535a4e44a4/items?countryCode=US")
            .with_status(200)
            .with_body("")
            .with_header("etag", "123457689")
            .create();
        let mock_update_playlist = mock("POST", "/playlists/7ce7df87-6d37-4465-80db-84535a4e44a4/items?countryCode=US")
            .with_status(200)
            .match_header("if-none-match", "123457689")
            .with_body(r#"{ "lastUpdated": 1600273268158, "addedItemIds": [ 79914999, 79915000 ] }"#)
            .create();

        let _result: Playlist = client().playlist_add_tracks("7ce7df87-6d37-4465-80db-84535a4e44a4", tracks, false).await.unwrap();
        mock_update_playlist.assert();
    }

    fn mock_request_success(method: &str, path: &str, body: &str) -> mockito::Mock {
        mock(method, path)
            .with_status(200)
            .with_body(body)
            .create()
    }

    fn mock_request_success_from_file(method: &str, path: &str, file_path: &str) -> mockito::Mock {
        mock(method, path)
            .with_status(200)
            .with_body_from_file(file_path)
            .create()
    }

    fn client() -> Tidal {
        Tidal::new(credential())
    }

    fn credential() -> TidalCredentials {
        let session: SessionInfo = SessionInfo {
            user_id: Some(1234),
            session_id: Some("session-id-1".to_owned()),
            country_code: "US".to_owned()
        };
        TidalCredentials {
            token: "some_token".to_owned(),
            session_info: Some(session)
        }
    }
}

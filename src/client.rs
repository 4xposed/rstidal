// Use 3rd party
use reqwest::header::HeaderMap;
use reqwest::{Client, Method, Response, StatusCode};
use serde::Deserialize;
use serde_json::Value;
use thiserror::Error;

#[cfg(test)]
use mockito;

// Use built-in library
use std::borrow::Cow;
use std::collections::HashMap;

// Use internal modules
use crate::model::artist::Artist;
use crate::model::album::Album;
use crate::auth::TidalCredentials;

// Possible errors returned from `rstidal` client.
#[derive(Debug, Error)]
pub enum ClientError {
    #[error("request unauthorized")]
    Unauthorized,
    #[error("tidal error: {0}")]
    Api(#[from] ApiError),
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

    async fn api_call(&self, method: Method, url: &str, payload: Option<&Value>) -> ClientResult<String> {
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

        let response = {
            let builder = self
                .client
                .request(method, &url.into_owned())
                .headers(headers);

            // Only add payload when sent
            let builder = if let Some(json) = payload {
                builder.json(json)
            } else {
                builder
            };

            builder.send().await.map_err(ClientError::from)?
        };

        if response.status().is_success() {
            response.text().await.map_err(Into::into)
        } else {
            Err(ClientError::from_response(response).await)
        }
    }

    pub async fn get(&self, url: &str, params: &mut HashMap<String, String>) -> ClientResult<String> {
        // Tidal's API requires countryCode to always be passed
        params.insert("countryCode".to_owned(), self.country_code());
        let param_string: String = serde_urlencoded::to_string(params).unwrap();
        let mut url_with_params = url.to_owned();
        url_with_params.push('?');
        url_with_params.push_str(&param_string);
        self.api_call(Method::GET, &url_with_params, None).await
    }

    //pub async fn post(&self, url: &str, payload: &Value) -> ClientResult<String> {
        //self.api_call(Method::POST, url, Some(payload)).await
    //}

    //pub async fn put(&self, url: &str, payload: &Value) -> ClientResult<String> {
        //self.api_call(Method::PUT, url, Some(payload)).await
    //}

    //pub async fn delete(&self, url: &str, payload: &Value) -> ClientResult<String> {
        //self.api_call(Method::DELETE, url, Some(payload)).await
    //}
    pub async fn artist(&self, id: &str) -> ClientResult<Artist> {
        let url = format!("/artists/{}", id);
        let result = self.get(&url, &mut HashMap::new()).await?;
        Self::convert_result::<Artist>(&result)
    }

    pub async fn album(&self, id: &str) -> ClientResult<Album> {
        let url = format!("/albums/{}", id);
        let result = self.get(&url, &mut HashMap::new()).await?;
        Self::convert_result::<Album>(&result)
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

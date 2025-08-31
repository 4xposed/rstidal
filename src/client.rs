// Use 3rd party
use log::debug;
use reqwest::header::HeaderMap;
use reqwest::{Client, Method, Response};
use serde::Deserialize;

#[cfg(test)]
use mockito;

// Use built-in library
use std::borrow::Cow;
use std::collections::HashMap;

// Use internal modules
use crate::auth::{Session, TidalCredentials};
use crate::config::{headers, params, Config};
use crate::error::{ClientError, SessionError, TidalError, TidalResult};
use crate::model::album::Album;
use crate::model::artist::Artist;
use crate::model::playlist::Playlist;
use crate::model::track::Track;

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

/// Main Tidal API client
pub struct Tidal {
    client: Client,
    pub(crate) credentials: TidalCredentials,
    config: Config,
}

impl Tidal {
    /// Create a new Tidal client with credentials and default configuration
    pub fn new(credentials: TidalCredentials) -> TidalResult<Self> {
        Self::with_config(credentials, Config::default())
    }

    /// Create a new Tidal client with custom configuration
    pub fn with_config(credentials: TidalCredentials, config: Config) -> TidalResult<Self> {
        if credentials.session.is_none() {
            return Err(TidalError::Session(SessionError::NoSession));
        }

        Ok(Self {
            client: Client::new(),
            credentials,
            config,
        })
    }

    /// Get the user ID from the current session
    pub fn user_id(&self) -> TidalResult<u32> {
        self.credentials
            .session
            .as_ref()
            .map(|session| session.user_id)
            .ok_or_else(|| TidalError::Session(SessionError::NoSession))
    }

    /// Get the session information
    pub fn session(&self) -> TidalResult<&Session> {
        self.credentials
            .session
            .as_ref()
            .ok_or_else(|| TidalError::Session(SessionError::NoSession))
    }

    async fn api_call(
        &self,
        method: Method,
        url: &str,
        query: Option<&HashMap<String, String>>,
        payload: Option<&HashMap<&str, &str>>,
        etag: Option<String>,
    ) -> TidalResult<Response> {
        #[cfg(not(test))]
        let base_url = self.config.base_url;
        #[cfg(test)]
        let base_url = &mockito::server_url();

        let mut url: Cow<str> = url.into();
        if !url.starts_with("http") {
            url = format!("{}{}", base_url, url).into();
        }

        let session = self.session()?;

        let mut headers = HeaderMap::new();

        // Use safe header parsing with proper error handling
        headers.insert(
            headers::X_TIDAL_SESSION_ID,
            session
                .session_id
                .parse()
                .map_err(|_| TidalError::Client(ClientError::InvalidResponse))?,
        );
        headers.insert(
            headers::ORIGIN,
            self.config
                .origin_header
                .parse()
                .map_err(|_| TidalError::Client(ClientError::InvalidResponse))?,
        );

        if let Some(etag) = etag {
            headers.insert(
                headers::IF_NONE_MATCH,
                etag.parse()
                    .map_err(|_| TidalError::Client(ClientError::EtagParse))?,
            );
        }

        // Tidal's API requires countryCode to always be passed
        let query_capacity = 1 + query.as_ref().map_or(0, |q| q.len());
        let mut query_params: HashMap<String, String> = HashMap::with_capacity(query_capacity);
        query_params.insert(
            params::COUNTRY_CODE.to_owned(),
            session.country_code.clone(),
        );

        if let Some(query) = query {
            query_params.extend(query.iter().map(|(k, v)| (k.clone(), v.clone())));
        }

        let builder = self
            .client
            .request(method, url.as_ref())
            .headers(headers)
            .query(&query_params);

        // Only add payload when provided
        let builder = if let Some(form) = payload {
            builder.form(form)
        } else {
            builder
        };

        debug!("Making API request to: {}", url);
        let response = builder
            .send()
            .await
            .map_err(|e| TidalError::Client(ClientError::Request(e)))?;

        debug!("API response status: {}", response.status());

        if response.status().is_success() {
            Ok(response)
        } else {
            Err(ClientError::from_response(response).await)
        }
    }

    /// Get the ETag header for a given URL
    pub async fn etag(&self, url: &str) -> TidalResult<String> {
        let response = self.api_call(Method::GET, url, None, None, None).await?;
        let headers = response.headers();

        headers
            .get(headers::ETAG)
            .ok_or_else(|| TidalError::Client(ClientError::EtagParse))?
            .to_str()
            .map(|s| s.to_owned())
            .map_err(|_| TidalError::Client(ClientError::EtagParse))
    }

    /// Make a GET request and return the response body as text
    pub async fn get(&self, url: &str, params: &HashMap<String, String>) -> TidalResult<String> {
        self.api_call(Method::GET, url, Some(params), None, None)
            .await?
            .text()
            .await
            .map_err(|e| TidalError::Client(ClientError::Request(e)))
    }

    /// Make a POST request with form data and return the response body as text
    pub async fn post(
        &self,
        url: &str,
        payload: &HashMap<&str, &str>,
        etag: Option<String>,
    ) -> TidalResult<String> {
        self.api_call(Method::POST, url, None, Some(payload), etag)
            .await?
            .text()
            .await
            .map_err(|e| TidalError::Client(ClientError::Request(e)))
    }

    /// Make a PUT request with form data and return the response body as text
    pub async fn put(
        &self,
        url: &str,
        payload: &HashMap<&str, &str>,
        etag: String,
    ) -> TidalResult<String> {
        self.api_call(Method::PUT, url, None, Some(payload), Some(etag))
            .await?
            .text()
            .await
            .map_err(|e| TidalError::Client(ClientError::Request(e)))
    }

    /// Convert a JSON string to the desired type
    pub fn convert_result<'a, T: Deserialize<'a>>(input: &'a str) -> TidalResult<T> {
        serde_json::from_str::<T>(input).map_err(|e| TidalError::Client(ClientError::JsonParse(e)))
    }
}

impl std::fmt::Debug for Tidal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tidal")
            .field("credentials", &self.credentials)
            .field("config", &self.config)
            .finish()
    }
}

// Endpoint implementations
use crate::endpoints::albums::Albums;
use crate::endpoints::artists::Artists;
use crate::endpoints::playlists::Playlists;
use crate::endpoints::search::Search;
use crate::endpoints::tracks::Tracks;

impl Tidal {
    /// Access album-related endpoints
    pub const fn albums(&self) -> Albums<'_> {
        Albums(self)
    }

    /// Access artist-related endpoints
    pub const fn artists(&self) -> Artists<'_> {
        Artists(self)
    }

    /// Access playlist-related endpoints
    pub const fn playlists(&self) -> Playlists<'_> {
        Playlists(self)
    }

    /// Access search endpoints
    pub const fn searches(&self) -> Search<'_> {
        Search(self)
    }

    /// Access track-related endpoints
    pub const fn tracks(&self) -> Tracks<'_> {
        Tracks(self)
    }
}

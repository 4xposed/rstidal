//! Configuration module for centralized settings and constants

/// API configuration constants
#[derive(Debug)]
pub struct Config {
    pub base_url: &'static str,
    pub login_url: &'static str,
    pub origin_header: &'static str,
    pub default_country_code: &'static str,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            base_url: "https://api.tidalhifi.com/v1",
            login_url: "https://api.tidalhifi.com/v1/login/username",
            origin_header: "http://listen.tidal.com",
            default_country_code: "US",
        }
    }
}

impl Config {
    /// Create a new configuration with custom settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a configuration for testing
    pub fn test(base_url: &str) -> Self {
        Self {
            base_url: Box::leak(base_url.to_string().into_boxed_str()),
            login_url: Box::leak(base_url.to_string().into_boxed_str()),
            origin_header: "http://listen.tidal.com",
            default_country_code: "US",
        }
    }
}

/// HTTP header constants
pub mod headers {
    pub const X_TIDAL_SESSION_ID: &str = "X-Tidal-SessionId";
    pub const ORIGIN: &str = "Origin";
    pub const IF_NONE_MATCH: &str = "If-None-Match";
    pub const ETAG: &str = "etag";
}

/// API endpoint constants
pub mod endpoints {
    pub const ARTISTS: &str = "/artists";
    pub const ALBUMS: &str = "/albums";
    pub const TRACKS: &str = "/tracks";
    pub const PLAYLISTS: &str = "/playlists";
    pub const SEARCH: &str = "/search";
    pub const USER_PLAYLISTS: &str = "/users/{user_id}/playlists";
}

/// Query parameter constants
pub mod params {
    pub const COUNTRY_CODE: &str = "countryCode";
    pub const QUERY: &str = "query";
    pub const LIMIT: &str = "limit";
    pub const TOKEN: &str = "token";
}

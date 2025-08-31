//! Comprehensive error handling for the Tidal API client

use reqwest::{Response, StatusCode};
use serde::Deserialize;
use thiserror::Error;

/// Main error type for all Tidal API operations
#[derive(Debug, Error)]
pub enum TidalError {
    #[error("Authentication failed: {0}")]
    Authentication(#[from] AuthError),

    #[error("Client error: {0}")]
    Client(#[from] ClientError),

    #[error("API error: {0}")]
    Api(#[from] ApiError),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Session error: {0}")]
    Session(#[from] SessionError),
}

/// Authentication-related errors
#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Authentication request failed")]
    RequestFailed(#[from] reqwest::Error),

    #[error("Invalid credentials provided")]
    InvalidCredentials,

    #[error("Token is missing or empty")]
    MissingToken,

    #[error("Failed to create session")]
    SessionCreationFailed,

    #[error("Session has expired")]
    SessionExpired,
}

/// Client-related errors
#[derive(Debug, Error)]
pub enum ClientError {
    #[error("HTTP request failed")]
    Request(#[from] reqwest::Error),

    #[error("JSON parsing failed: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("URL encoding failed: {0}")]
    UrlEncode(#[from] serde_urlencoded::ser::Error),

    #[error("ETag header parsing failed")]
    EtagParse,

    #[error("HTTP status code: {0}")]
    StatusCode(StatusCode),

    #[error("Response body is empty")]
    EmptyResponse,

    #[error("Invalid response format")]
    InvalidResponse,
}

/// Session-related errors
#[derive(Debug, Error)]
pub enum SessionError {
    #[error("No active session found")]
    NoSession,

    #[error("Session is invalid")]
    InvalidSession,

    #[error("User ID is missing from session")]
    MissingUserId,

    #[error("Country code is missing from session")]
    MissingCountryCode,
}

/// API-specific errors returned by Tidal
#[derive(Debug, Error, Deserialize)]
#[serde(untagged)]
pub enum ApiError {
    #[error("API error {status}: {message}")]
    Standard {
        status: u16,
        #[serde(alias = "userMessage")]
        message: String,
        #[serde(alias = "subStatus")]
        sub_status: Option<u16>,
    },

    #[error("Unknown API error: {0}")]
    Unknown(String),
}

impl ClientError {
    /// Create a ClientError from an HTTP response
    pub async fn from_response(response: Response) -> TidalError {
        let status = response.status();
        match status {
            StatusCode::UNAUTHORIZED => TidalError::Authentication(AuthError::InvalidCredentials),
            StatusCode::FORBIDDEN | StatusCode::NOT_FOUND => {
                // Try to parse as API error first
                match response.json::<ApiError>().await {
                    Ok(api_error) => TidalError::Api(api_error),
                    Err(_) => TidalError::Client(ClientError::StatusCode(status)),
                }
            }
            status => TidalError::Client(ClientError::StatusCode(status)),
        }
    }
}

impl From<StatusCode> for ClientError {
    fn from(code: StatusCode) -> Self {
        Self::StatusCode(code)
    }
}

/// Result type alias for convenience
pub type TidalResult<T> = Result<T, TidalError>;

/// Helper functions for error handling
impl TidalError {
    /// Check if the error is recoverable (e.g., temporary network issues)
    pub fn is_recoverable(&self) -> bool {
        match self {
            TidalError::Client(ClientError::Request(_)) => true,
            TidalError::Client(ClientError::StatusCode(status)) => status.is_server_error(),
            _ => false,
        }
    }

    /// Check if the error requires re-authentication
    pub fn requires_reauth(&self) -> bool {
        matches!(
            self,
            TidalError::Authentication(_)
                | TidalError::Session(SessionError::InvalidSession)
                | TidalError::Session(SessionError::NoSession)
        )
    }

    /// Get a user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            TidalError::Authentication(AuthError::InvalidCredentials) => {
                "Please check your username and password".to_string()
            }
            TidalError::Authentication(AuthError::MissingToken) => {
                "Application token is required".to_string()
            }
            TidalError::Session(SessionError::NoSession) => "Please log in to continue".to_string(),
            TidalError::Client(ClientError::Request(_)) => {
                "Network connection failed. Please try again".to_string()
            }
            _ => self.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_is_recoverable() {
        let error = TidalError::Client(ClientError::StatusCode(StatusCode::INTERNAL_SERVER_ERROR));
        assert!(error.is_recoverable());

        let error = TidalError::Authentication(AuthError::InvalidCredentials);
        assert!(!error.is_recoverable());
    }

    #[test]
    fn test_error_requires_reauth() {
        let error = TidalError::Authentication(AuthError::SessionExpired);
        assert!(error.requires_reauth());

        let error = TidalError::Client(ClientError::JsonParse(
            serde_json::from_str::<()>("invalid").unwrap_err(),
        ));
        assert!(!error.requires_reauth());
    }

    #[test]
    fn test_user_friendly_messages() {
        let error = TidalError::Authentication(AuthError::InvalidCredentials);
        assert_eq!(
            error.user_message(),
            "Please check your username and password"
        );

        let error = TidalError::Session(SessionError::NoSession);
        assert_eq!(error.user_message(), "Please log in to continue");
    }
}

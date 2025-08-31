// Use 3rd party
use log::{debug, error};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[cfg(test)]
use mockito;

// Use built-in library
use std::collections::HashMap;

// Use internal modules
use crate::config::params;
use crate::error::{AuthError, TidalError, TidalResult};

#[derive(Clone, Debug)]
pub struct TidalCredentials {
    pub token: String,
    pub session: Option<Session>,
}

impl TidalCredentials {
    #[must_use]
    pub fn new(token: &str) -> Self {
        Self {
            token: token.to_owned(),
            session: None,
        }
    }

    pub fn session(mut self, session: Option<Session>) -> Self {
        self.session = session;
        self
    }

    /// Create a session with the provided credentials
    /// Returns an error if the token is empty or session creation fails
    pub async fn create_session(self, username: &str, password: &str) -> TidalResult<Self> {
        if self.token.is_empty() {
            return Err(TidalError::Authentication(AuthError::MissingToken));
        }

        let session = Session::get_session(&self.token, username, password).await?;
        Ok(self.session(Some(session)))
    }
}

//Tidal session example:
//{
//"userId": 173393989,
//"sessionId": "84df94d0-9t0b-537a-a485-4404e45581ft",
//"countryCode": "DE"
//}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub user_id: u32,
    pub session_id: String,
    pub country_code: String,
}

impl Session {
    pub async fn get_session(token: &str, username: &str, password: &str) -> TidalResult<Self> {
        let mut payload: HashMap<&str, &str> = HashMap::with_capacity(2);
        payload.insert("username", username);
        payload.insert("password", password);
        Self::fetch_session_data(token, &payload).await
    }

    async fn fetch_session_data(token: &str, payload: &HashMap<&str, &str>) -> TidalResult<Self> {
        let client = Client::new();
        let query = [(params::TOKEN, token)];

        #[cfg(not(test))]
        let url = "https://api.tidalhifi.com/v1/login/username";

        #[cfg(test)]
        let url = &mockito::server_url();

        let response = client
            .post(url)
            .query(&query)
            .form(payload)
            .send()
            .await
            .map_err(AuthError::RequestFailed)?;

        if response.status().is_success() {
            debug!("Session creation successful");
            let session: Session = response
                .json()
                .await
                .map_err(|e| TidalError::Authentication(AuthError::RequestFailed(e)))?;
            Ok(session)
        } else {
            error!(
                "Session creation failed. Status: {}, Token length: {}, Payload: {:?}",
                response.status(),
                token.len(),
                payload.keys().collect::<Vec<_>>()
            );

            match response.status() {
                reqwest::StatusCode::UNAUTHORIZED => {
                    Err(TidalError::Authentication(AuthError::InvalidCredentials))
                }
                _ => Err(TidalError::Authentication(AuthError::SessionCreationFailed)),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::mock;

    #[test]
    fn test_credential_set_new() {
        let credentials = TidalCredentials::new("some_token");
        assert_eq!(credentials.token, "some_token".to_owned());
    }

    #[test]
    fn test_credential_set_session_info() {
        let session = Session {
            user_id: 1234,
            session_id: "xq123".to_owned(),
            country_code: "US".to_owned(),
        };
        let credentials = TidalCredentials::new("some_token").session(Some(session));
        assert_eq!(credentials.session.is_some(), true);
    }

    #[tokio::test]
    async fn test_credential_create_session_w_token() {
        let token = "some_token";
        let username = "myuser@example.com";
        let password = "somepassword";
        let credentials = TidalCredentials::new(token);

        // Test successful login
        {
            let _mock = mock_successful_login();
            let credential_w_session = credentials
                .clone()
                .create_session(username, password)
                .await
                .unwrap();
            assert_eq!(
                credential_w_session.session.unwrap().session_id,
                "session-id-123"
            );
        }
        // Test failed login
        {
            let _mock = mock_failed_login();
            let credential_result = credentials.clone().create_session(username, password).await;
            assert!(credential_result.is_err());
        }
    }

    #[tokio::test]
    async fn test_create_session_empty_token() {
        let credentials = TidalCredentials::new("");
        let result = credentials.create_session("user", "pass").await;
        assert!(matches!(
            result,
            Err(TidalError::Authentication(AuthError::MissingToken))
        ));
    }

    fn mock_successful_login() -> mockito::Mock {
        mock("POST", "/?token=some_token")
            .with_status(200)
            .with_body(r#"{"userId": 123, "sessionId": "session-id-123", "countryCode": "US"}"#)
            .create()
    }

    fn mock_failed_login() -> mockito::Mock {
        mock("POST", "/?token=some_token")
            .with_status(401)
            .with_body(
                r#"{"status": 401, "subStatus": 3001, "userMessage": "Invalid credentials"}"#,
            )
            .create()
    }
}

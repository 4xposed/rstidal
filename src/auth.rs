// Use 3rd party
use log::{debug, error};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[cfg(test)]
use mockito;

// Use built-in library
use std::collections::HashMap;

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

    #[must_use]
    pub async fn create_session(self, username: &str, password: &str) -> Self {
        if self.token.is_empty() {
            // A token needs to be set before this function can be called
            panic!("Application Token needs to be set")
        }
        let token = self.token.to_owned();
        let session = Session::get_session(&token, username, password).await.ok();
        self.session(session)
    }
}

//Tidal session example:
//{
    //"userId": 173393989,
    //"sessionId": "84df94d0-9t0b-537a-a485-4404e45581ft",
    //"countryCode": "DE"
//}

#[derive(thiserror::Error,Debug)]
pub enum AuthError {
    #[error("The Authe request Failed")]
    AuthRequestFailed { #[from] source: reqwest::Error },
    #[error("Fetch session failed")]
    CreateSessionFailed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub user_id: u32,
    pub session_id: String,
    pub country_code: String,
}

impl Session {
    pub async fn get_session(token: &str, username: &str, password: &str) -> Result<Self, AuthError> {
        let mut payload: HashMap<&str, &str> = HashMap::new();
        payload.insert("username", username);
        payload.insert("password", password);
        Self::fetch_session_data(token, &payload).await
    }

    async fn fetch_session_data(token: &str, payload: &HashMap<&str, &str>) -> Result<Self, AuthError> {
        let client = Client::new();
        let token = token.to_owned();
        let query = [("token", &token)];

        #[cfg(not(test))]
        let url = "https://api.tidalhifi.com/v1/login/username";

        #[cfg(test)]
        let url = &mockito::server_url();

        let response = client
            .post(url)
            .query(&query)
            .form(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            debug!("response content: {:?}", response);
            let session: Session = response.json().await?;
            Ok(session)
        } else {
            error!(
                "Creating session failed. token: {:?}, form: {:?}",
                &token, &payload
            );
            error!("{:?}", response);
            Err(AuthError::CreateSessionFailed)
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
        let password = "somepawssowrd";
        let credentials = TidalCredentials::new(token);

        // Test scucessful login
        {
            let _mock = mock_successful_login();
            let credential_w_session = credentials.clone().create_session(username, password).await;
            assert_eq!(
                credential_w_session.session.unwrap().session_id,
                "session-id-123"
            );
        }
        // Test failed login
        {
            let _mock = mock_failed_login();
            let credential_wo_session =
                credentials.clone().create_session(username, password).await;
            assert_eq!(credential_wo_session.session.is_none(), true);
        }
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

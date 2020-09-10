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
    pub session_info: Option<SessionInfo>
}

impl TidalCredentials {
    #[must_use]
    pub fn new(token: &str) -> Self {
        Self {
            token: token.to_owned(),
            session_info: None
        }
    }

    pub fn session_info(mut self, sesion_info: SessionInfo) -> Self {
        self.session_info = Some(sesion_info);
        self
    }

    #[must_use]
    pub async fn create_session(self, username: &str, password: &str) -> Self {
        if self.token.is_empty() {
            // A token needs to be set before this function can be called
            panic!("Application Token needs to be set")
        }
        let token = self.token.to_owned();
        let mut session_info = match self.session_info {
            Some(ref session_info) => session_info.to_owned(),
            None => SessionInfo::new()
        };

        let session = session_info.get_session(
            &token,
            username,
            password
        ).await;

        if let Some(session) = session {
            session_info = session
        } else {
            error!("Invalid credentials or Application Token")
        }

        self.session_info(session_info)
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
pub struct SessionInfo {
    pub user_id: Option<u32>,
    pub session_id: Option<String>,
    pub country_code: String
}

impl SessionInfo {
    pub fn new() -> Self {
        Self {
            user_id: None,
            session_id: None,
            country_code: String::new()
        }
    }

    pub async fn get_session(&self, token: &str, username: &str, password: &str) -> Option<Self> {
        let mut payload: HashMap<&str, &str> = HashMap::new();
        payload.insert("username", username);
        payload.insert("password", password);
        self.fetch_session_data(token, &payload).await
    }

    async fn fetch_session_data(&self, token: &str, payload: &HashMap<&str, &str>) -> Option<Self> {
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
            .await
            .expect("Login request failed");

        if response.status().is_success() {
            debug!("response content: {:?}", response);
            let session_info: SessionInfo = response
                .json()
                .await
                .expect("Error parsing session_info");
            Some(session_info)
        } else {
            error!("fetch session failed. token: {:?}, form: {:?}", &token, &payload);
            error!("{:?}", response);
            None
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
        let session_info = SessionInfo::new();
        let credentials = TidalCredentials::new("some_token").session_info(session_info);
        assert_eq!(credentials.session_info.is_some(), true);
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
            assert_eq!(credential_w_session.session_info.unwrap().session_id.is_some(), true);
        }
        // Test failed login
        {
            let _mock = mock_failed_login();
            let credential_wo_session = credentials.clone().create_session(username, password).await;
            assert_eq!(credential_wo_session.session_info.unwrap().session_id.is_some(), false);
        }
    }


    #[test]
    fn test_session_info_default() {
        let session_info = SessionInfo::new();
        assert_eq!(session_info.user_id.is_none(), true);
        assert_eq!(session_info.session_id.is_none(), true);
        assert_eq!(session_info.country_code, String::new());
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
            .with_body(r#"{"status": 401, "subStatus": 3001, "userMessage": "Invalid credentials"}"#)
            .create()
    }

}

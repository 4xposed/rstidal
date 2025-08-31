//! Integration tests for the Tidal API client

use mockito::{mock, Matcher};
use rstidal::auth::{Session, TidalCredentials};
use rstidal::client::Tidal;
use rstidal::config::Config;
use rstidal::error::{AuthError, SessionError, TidalError};

// Test helper functions
fn test_client() -> Tidal {
    let session = Session {
        user_id: 1234,
        session_id: "test-session-id".to_owned(),
        country_code: "US".to_owned(),
    };

    let credentials = TidalCredentials::new("test-token").session(Some(session));
    let config = Config::test(&mockito::server_url());

    Tidal::with_config(credentials, config).expect("Failed to create test client")
}

fn mock_request_success_from_file(
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

fn country_code_matcher() -> Matcher {
    Matcher::UrlEncoded("countryCode".into(), "US".into())
}

fn query_matcher(key: &str, value: &str) -> Matcher {
    Matcher::UrlEncoded(key.into(), value.into())
}

#[tokio::test]
async fn test_client_creation_with_session() {
    let session = Session {
        user_id: 123,
        session_id: "test-session".to_owned(),
        country_code: "US".to_owned(),
    };
    let credentials = TidalCredentials::new("test-token").session(Some(session));
    let client = Tidal::new(credentials);
    assert!(client.is_ok());
}

#[tokio::test]
async fn test_client_creation_without_session() {
    let credentials = TidalCredentials::new("test-token");
    let client = Tidal::new(credentials);
    match client {
        Err(TidalError::Session(SessionError::NoSession)) => {
            // This is expected
        }
        _ => panic!("Expected SessionError::NoSession"),
    }
}

#[tokio::test]
async fn test_user_id_retrieval() {
    let client = test_client();
    let user_id = client.user_id().unwrap();
    assert_eq!(user_id, 1234);
}

#[tokio::test]
async fn test_session_retrieval() {
    let client = test_client();
    let session = client.session().unwrap();
    assert_eq!(session.user_id, 1234);
    assert_eq!(session.country_code, "US");
}

#[tokio::test]
async fn test_artist_get() {
    let _mock = mock_request_success_from_file(
        "GET",
        "/artists/37312",
        vec![country_code_matcher()],
        "tests/files/artist.json",
    );

    let client = test_client();
    let result = client.artists().get("37312").await.unwrap();

    assert_eq!(result.id, Some(37312));
    assert_eq!(result.name, Some("myband".to_owned()));
}

#[tokio::test]
async fn test_search_functionality() {
    let _mock = mock_request_success_from_file(
        "GET",
        "/search",
        vec![
            country_code_matcher(),
            query_matcher("query", "trivium"),
            query_matcher("limit", "10"),
        ],
        "tests/files/search.json",
    );

    let client = test_client();
    let result = client.searches().find("trivium", None).await.unwrap();

    assert_eq!(result.artists.items.len(), 10);
    assert_eq!(result.albums.items.len(), 10);
    assert_eq!(result.tracks.items.len(), 10);
    assert_eq!(result.playlists.items.len(), 10);
}

#[tokio::test]
async fn test_error_handling_unauthorized() {
    let _mock = mock("GET", "/artists/37312")
        .match_query(country_code_matcher())
        .with_status(401)
        .with_body(r#"{"status": 401, "userMessage": "Unauthorized"}"#)
        .create();

    let client = test_client();
    let result = client.artists().get("37312").await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        TidalError::Authentication(AuthError::InvalidCredentials)
    ));
}

#[tokio::test]
async fn test_etag_functionality() {
    let _mock = mock("GET", "/test-endpoint")
        .match_query(country_code_matcher())
        .with_status(200)
        .with_header("etag", "test-etag-value")
        .with_body("")
        .create();

    let client = test_client();
    let etag = client.etag("/test-endpoint").await.unwrap();

    assert_eq!(etag, "test-etag-value");
}

// Authentication tests have been moved to unit tests in the auth module
// since they require special mock server configuration

#[tokio::test]
async fn test_empty_token_error() {
    let credentials = TidalCredentials::new("");
    let result = credentials.create_session("user", "pass").await;

    assert!(matches!(
        result.unwrap_err(),
        TidalError::Authentication(AuthError::MissingToken)
    ));
}

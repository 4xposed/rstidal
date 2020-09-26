//! Endpoint functions related to playlists

use std::collections::HashMap;

use crate::client::*;
use crate::model::playlist::*;
use crate::model::track::*;

pub struct Playlists<'a>(pub &'a Tidal);

impl Playlists<'_> {
    pub async fn get(&self, id: &str) -> ClientResult<Playlist> {
        let url = format!("/playlists/{}", id);
        let result = self.0.get(&url, &mut HashMap::new()).await?;
        Tidal::convert_result::<Playlist>(&result)
    }

    pub async fn search(&self, term: &str, limit: Option<u16>) -> ClientResult<Vec<Playlist>> {
        let playlists = self.0.search(term, limit).await?.playlists.items;
        Ok(playlists)
    }

    pub async fn tracks(&self, id: &str) -> ClientResult<Vec<Track>> {
        let url = format!("/playlists/{}/tracks", id);
        let result = self.0.get(&url, &mut HashMap::new()).await?;
        let tracks = Tidal::convert_result::<TidalItems<Track>>(&result)?.items;
        Ok(tracks)
    }

    pub async fn create(&self, title: &str, description: &str) -> ClientResult<Playlist> {
        let user_id = self.0.user_id();
        let url = format!("/users/{}/playlists", user_id);
        println!("URL: {:?}", url);
        let mut form: HashMap<&str, &str> = HashMap::new();
        form.insert("title", title);
        form.insert("description", description);
        let result = self.0.post(&url, &form, None).await?;
        Tidal::convert_result::<Playlist>(&result)
    }

    pub async fn add_tracks(
        &self,
        id: &str,
        tracks: Vec<Track>,
        add_dupes: bool,
    ) -> ClientResult<Playlist> {
        let url = format!("/playlists/{}/items", id);

        // Get etag for the Playlist to be allowed to update the Playlist
        let etag: String = self.0.etag(&url).await?;

        // Convert the list of Tracks to a String with comma separated Track IDs
        let track_ids: Vec<String> = tracks
            .iter()
            .map(|track| track.id.expect("Track struct must have an ID").to_string())
            .collect();
        let track_ids: String = track_ids.join(",");

        let on_dupes: String = if add_dupes {
            "ADD".to_owned()
        } else {
            "FAIL".to_owned()
        };

        let mut form: HashMap<&str, &str> = HashMap::new();
        form.insert("trackIds", &track_ids);
        form.insert("onDupes", &on_dupes);

        // Submit request to add the Tracks to the Playlist
        self.0.post(&url, &form, Some(etag)).await?;

        // Get updated Playlist
        self.0.playlist(id).await
    }

    pub async fn user_playlists(&self) -> ClientResult<Vec<Playlist>> {
        let user_id = self.0.user_id();
        let url = format!("/users/{}/playlists", user_id);
        let result = self.0.get(&url, &mut HashMap::new()).await?;
        let playlists = Tidal::convert_result::<TidalItems<Playlist>>(&result)?.items;
        Ok(playlists)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::tests::{client, mock_request_success_from_file};
    use mockito::{mock, Matcher};

    #[tokio::test]
    async fn get() {
        let _mock = mock_request_success_from_file(
            "GET",
            "/playlists/7ce7df87-6d37-4465-80db-84535a4e44a4",
            vec![Matcher::UrlEncoded("countryCode".into(), "US".into())],
            "tests/files/playlist.json",
        );

        let result: Playlist = client()
            .playlists()
            .get("7ce7df87-6d37-4465-80db-84535a4e44a4")
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
    async fn create() {
        let _mock = mock_request_success_from_file(
            "POST",
            "/users/1234/playlists",
            vec![Matcher::UrlEncoded("countryCode".into(), "US".into())],
            "tests/files/create_playlist.json",
        );

        let result: Playlist = client()
            .playlists()
            .create("something", "some desc")
            .await
            .unwrap();

        assert_eq!(result.title.unwrap(), "something".to_string());
        assert_eq!(result.description.unwrap(), "some desc".to_string());
    }

    #[tokio::test]
    async fn user_playlists() {
        let _mock = mock_request_success_from_file(
            "GET",
            "/users/1234/playlists",
            vec![Matcher::UrlEncoded("countryCode".into(), "US".into())],
            "tests/files/user_playlists.json",
        );

        let result: Vec<Playlist> = client().playlists().user_playlists().await.unwrap();
        let expected_result = Playlist {
            uuid: Some("8edf5a89-fec4-4aa3-80ab-9e00a83633a2".to_owned()),
            title: Some("roadtrip".to_owned()),
            ..Default::default()
        };
        assert_eq!(result[0].uuid, expected_result.uuid);
        assert_eq!(result[0].title, expected_result.title);
    }

    #[tokio::test]
    async fn tracks() {
        let _mock = mock_request_success_from_file(
            "GET",
            "/playlists/7ce7df87-6d37-4465-80db-84535a4e44a4/tracks",
            vec![Matcher::UrlEncoded("countryCode".into(), "US".into())],
            "tests/files/playlist_tracks.json",
        );

        let result: Vec<Track> = client()
            .playlists()
            .tracks("7ce7df87-6d37-4465-80db-84535a4e44a4")
            .await
            .unwrap();
        let expected_first_result = Track {
            title: Some("FULL OF HEALTH".to_owned()),
            ..Default::default()
        };
        assert_eq!(result[0].title, expected_first_result.title);
    }

    #[tokio::test]
    async fn add_tracks() {
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
            .playlists()
            .add_tracks("7ce7df87-6d37-4465-80db-84535a4e44a4", tracks, false)
            .await
            .unwrap();
        mock_update_playlist.assert();
    }
}

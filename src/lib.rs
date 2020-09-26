//! Rstidal is a wrapper for the Tidal API.
//!
//! ## Configuration
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! rstidal = "0.1.2"
//! ```
//!
//! By default, Rstidal uses asynchronous programming with `asycn` and `await`.
//!
//! ## Getting Started
//!
//! ## Authorization
//!
//! Since all methods required user authentication, you are required to create a
//! session using a Tidal username and password.
//! In order to authenticate a user your application needs an Application Token.
//!
//!
//! ## How to get an Application Token
//!
//! Using a debug proxy (Charles or Fiddler) open your Tidal Desktop application, look for
//! requests to `api.tidal.com` and copy the value it uses in the header `X-Tidal-Token`.
//!
//! ### Examples
//!
//! ```toml
//! [dependencies]
//! rstidal = { version = "0.1.0" }
//! tokio = { version = "0.2", feeatures = ["full"] }
//! ```
//!
//! ```rust
//! use rstidal::client::Tidal;
//! use rstidal::auth::TidalCredentials;
//! use dotenv::dotenv;
//! use std::env;
//!
//! #[tokio::main]
//! async fn main() {
//!     {
//!         dotenv().ok();
//!     }
//!
//!     // Set the token aquired by inspecting your Tidal Desktop application.
//!     let token = env::var("RSTIDAL_APP_TOKEN").unwrap();
//!     let credentials = TidalCredentials::new(&token);
//!
//!     // Create a session using your user credentials.
//!     let username = env::var("RSTIDAL_USERNAME").unwrap();
//!     let password = env::var("RSTIDAL_PASSWORD").unwrap();
//!     let credentials = credentials.create_session(&username, &password).await;
//!
//!     // Use the credentials to start the client
//!     let client = Tidal::new(credentials);
//!     let artist = client.artists().get("37312").await;
//!     println!("{:?}", artist.unwrap());
//! }
//! ```

pub mod auth;
pub mod client;
pub mod endpoints;
pub mod model;

//! # Rstidal - Asynchronous Tidal API Client
//!
//! Rstidal is a modern, fully asynchronous Rust client for the Tidal music streaming API.
//! It provides comprehensive access to Tidal's music catalog including artists, albums,
//! tracks, and playlists with a clean, type-safe interface.
//!
//! ## Features
//!
//! - **Fully Async**: Built with `tokio` and `reqwest` for high-performance async operations
//! - **Type Safety**: Comprehensive error handling with custom error types
//! - **Clean API**: Organized endpoint modules for different resource types
//! - **Robust Error Handling**: Detailed error types with user-friendly messages
//! - **Session Management**: Automatic session handling and authentication
//! - **Testing Support**: Built-in support for mocking and testing
//!
//! ## Quick Start
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! rstidal = "0.1.2"
//! tokio = { version = "1.0", features = ["full"] }
//! ```
//!
//! ## Authentication
//!
//! All API methods require user authentication. You need:
//! 1. An Application Token (from Tidal Desktop app)
//! 2. Valid Tidal user credentials
//!
//! ### Getting an Application Token
//!
//! 1. Use a debug proxy (Charles, Fiddler, or browser dev tools)
//! 2. Open your Tidal Desktop application
//! 3. Look for requests to `api.tidal.com`
//! 4. Copy the `X-Tidal-Token` header value
//!
//! ## Usage Examples
//!
//! ### Basic Usage
//!
//! ```rust,no_run
//! use rstidal::{TidalCredentials, Tidal, TidalResult};
//! use std::env;
//!
//! #[tokio::main]
//! async fn main() -> TidalResult<()> {
//!     // Get credentials from environment
//!     let token = env::var("TIDAL_TOKEN").expect("TIDAL_TOKEN not set");
//!     let username = env::var("TIDAL_USERNAME").expect("TIDAL_USERNAME not set");
//!     let password = env::var("TIDAL_PASSWORD").expect("TIDAL_PASSWORD not set");
//!
//!     // Create session
//!     let credentials = TidalCredentials::new(&token)
//!         .create_session(&username, &password)
//!         .await?;
//!
//!     // Initialize client
//!     let client = Tidal::new(credentials)?;
//!
//!     // Search for an artist
//!     let artists = client.artists().search("Radiohead", Some(5)).await?;
//!     println!("Found {} artists", artists.len());
//!
//!     // Get artist details
//!     if let Some(artist) = artists.first() {
//!         if let Some(id) = artist.id {
//!             let albums = client.artists().albums(&id.to_string()).await?;
//!             println!("Artist has {} albums", albums.len());
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Advanced Usage with Error Handling
//!
//! ```rust,no_run
//! use rstidal::{TidalCredentials, Tidal, TidalError, TidalResult};
//!
//! #[tokio::main]
//! async fn main() {
//!     match run_example().await {
//!         Ok(_) => println!("Success!"),
//!         Err(e) => {
//!             eprintln!("Error: {}", e.user_message());
//!             if e.requires_reauth() {
//!                 eprintln!("Please check your credentials and try again.");
//!             }
//!         }
//!     }
//! }
//!
//! async fn run_example() -> TidalResult<()> {
//!     let credentials = TidalCredentials::new("your-token")
//!         .create_session("username", "password")
//!         .await?;
//!
//!     let client = Tidal::new(credentials)?;
//!
//!     // Get user's playlists
//!     let playlists = client.playlists().user_playlists().await?;
//!
//!     for playlist in playlists {
//!         if let Some(title) = &playlist.title {
//!             println!("Playlist: {}", title);
//!
//!             // Get tracks in playlist
//!             if let Some(uuid) = &playlist.uuid {
//!                 let tracks = client.playlists().tracks(uuid).await?;
//!                 println!("  {} tracks", tracks.len());
//!             }
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## API Structure
//!
//! The API is organized into modules for different resource types:
//!
//! - `client.artists()` - Artist-related operations
//! - `client.albums()` - Album-related operations
//! - `client.tracks()` - Track-related operations
//! - `client.playlists()` - Playlist-related operations
//! - `client.searches()` - Search operations across all types
//!
//! ## Error Handling
//!
//! Rstidal provides comprehensive error handling with the `TidalError` enum:
//!
//! ```rust,no_run
//! use rstidal::{Tidal, TidalCredentials, TidalError};
//! use rstidal::error::{AuthError, ClientError};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let credentials = TidalCredentials::new("token");
//! # let client = Tidal::new(credentials)?;
//! match client.artists().get("invalid-id").await {
//!     Ok(artist) => println!("Artist: {:?}", artist),
//!     Err(TidalError::Authentication(AuthError::InvalidCredentials)) => {
//!         println!("Please check your login credentials");
//!     },
//!     Err(TidalError::Client(ClientError::StatusCode(status))) => {
//!         println!("HTTP error: {}", status);
//!     },
//!     Err(e) => println!("Other error: {}", e),
//! }
//! # Ok(())
//! # }
//! ```

pub mod auth;
pub mod client;
pub mod config;
pub mod endpoints;
pub mod error;
pub mod model;

// Re-export commonly used types for convenience
pub use auth::{Session, TidalCredentials};
pub use client::Tidal;
pub use error::{TidalError, TidalResult};

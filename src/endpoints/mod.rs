pub mod albums;
pub mod artists;
pub mod playlists;
pub mod search;
pub mod tracks;

use crate::client::Tidal;
use crate::endpoints::albums::*;
use crate::endpoints::artists::*;
use crate::endpoints::playlists::*;
use crate::endpoints::search::*;
use crate::endpoints::tracks::*;

// Endpoint function namespaces

impl Tidal {
    pub const fn albums(&self) -> Albums {
        Albums(&self)
    }

    pub const fn artists(&self) -> Artists {
        Artists(&self)
    }

    pub const fn playlists(&self) -> Playlists {
        Playlists(&self)
    }

    pub const fn searches(&self) -> Search {
        Search(&self)
    }

    pub const fn tracks(&self) -> Tracks {
        Tracks(&self)
    }
}

use std::sync::Arc;

use snafu::{ResultExt, Snafu};
use tidalrs::{TidalClient, Track};
use tracing::instrument;

pub use tidalrs::{Album, FavoriteAlbum, AlbumSummary, ArtistSummary, AlbumType,
    Playlist, PlaylistCreator};

pub struct AlbumService {
    tidal_client: Arc<TidalClient>,
}

impl AlbumService {
    #[instrument(skip(tidal_client))]
    pub fn new(tidal_client: Arc<TidalClient>) -> Self {
        Self {
            tidal_client
        }
    }

    /// Get the users favourite albums. FavouriteAlbum contains the Album, aswell as the date it
    /// was favourited.
    #[instrument(skip(self), err)]
    pub async fn favourite_albums(&self) -> Result<Vec<FavoriteAlbum>, AlbumServiceError> {
        // TODO: Pages, sorting
        Ok(self.tidal_client.favorite_albums(None, None, None, None).await
            .context(FavouriteAlbumsSnafu)?.items)
    }

    /// Get the tracks from an album
    #[instrument(skip(self), err)]
    pub async fn album_tracks(&self, id: u64) -> Result<Vec<Track>, AlbumServiceError> {
        Ok(self.tidal_client.album_tracks(id, None, None).await
            .context(AlbumTracksSnafu { id })?.items)
    }
    
    /// Get the users playlists
    #[instrument(skip(self), err)]
    pub async fn user_playlists(&self) -> Result<Vec<Playlist>, AlbumServiceError> {
        // TODO: Pages, sorting
        Ok(self.tidal_client.user_playlists(None, None).await
            .context(UserPlaylistsSnafu)?.items)
    }

    /// Get the tracks from a playlist
    #[instrument(skip(self), err)]
    pub async fn playlist_tracks(&self, id: &str) -> Result<Vec<Track>, AlbumServiceError> {
        Ok(self.tidal_client.playlist_tracks(id, None, None).await
            .context(PlaylistTracksSnafu { id })?.items)
    }
}

#[derive(Debug, Snafu)]
pub enum AlbumServiceError {
    #[snafu(display("Could not fetch favorite_albums"))]
    FavouriteAlbums {
        source: tidalrs::Error,
    },
    #[snafu(display("Could not fetch user playlists"))]
    UserPlaylists {
        source: tidalrs::Error,
    },
    #[snafu(display("Could not get Albums (#{id}) tracks"))]
    AlbumTracks {
        source: tidalrs::Error,
        id: u64,
    },
    #[snafu(display("Could not get playlist (#{id}) tracks"))]
    PlaylistTracks {
        source: tidalrs::Error,
        id: String,
    },
}

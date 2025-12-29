use serde::{Deserialize, Serialize};
use specta::Type;
use structural_convert::StructuralConvert;
use tideperfect::services::album::{Playlist, PlaylistCreator};

use crate::dtos::album::ArtistSummaryDTO;

/// Represents a playlist from the Tidal catalog.
///
/// This structure contains all available information about a playlist,
/// including metadata, statistics, and modification capabilities.
#[derive(Default, Debug, Serialize, Deserialize, Clone, StructuralConvert, Type)]
#[serde(rename_all = "camelCase")]
#[convert(from(Playlist))]
#[convert(into(Playlist))]
pub struct PlaylistDTO {
    /// Unique playlist identifier (UUID format)
    pub uuid: String,
    /// Playlist title
    pub title: String,
    /// Tidal URL for the playlist
    #[serde(default)]
    pub url: Option<String>,
    /// Information about the playlist creator
    pub creator: PlaylistCreatorDTO,
    /// Playlist description
    #[serde(default)]
    pub description: String,

    /// Total number of tracks in the playlist
    pub number_of_tracks: u32,
    /// Total number of videos in the playlist
    pub number_of_videos: u32,
    /// Total duration of the playlist in seconds
    pub duration: u32,
    /// Popularity score for the playlist
    pub popularity: u32,

    /// ISO timestamp when the playlist was last updated
    pub last_updated: String,
    /// ISO timestamp when the playlist was created
    pub created: String,
    /// ISO timestamp when the last item was added to the playlist
    pub last_item_added_at: Option<String>,

    /// Type of playlist (e.g., "USER", "EDITORIAL")
    #[serde(rename = "type")]
    pub playlist_type: Option<String>,
    /// Whether the playlist is publicly visible
    pub public_playlist: bool,
    /// Playlist cover image identifier
    ///
    /// Use image_url() to get the full URL of the image
    pub image: Option<String>,
    /// Square version of the playlist cover image
    ///
    /// Use square_image_url() to get the full URL of the square image
    pub square_image: Option<String>,
    /// Custom image URL for the playlist
    pub custom_image_url: Option<String>,
    /// Artists promoted in this playlist
    pub promoted_artists: Option<Vec<ArtistSummaryDTO>>,

    /// ETag for concurrency control when modifying the playlist
    ///
    /// This is needed for adding or removing tracks from the playlist
    pub etag: Option<String>,
}

/// Information about the creator of a playlist.
///
/// This structure contains details about who created the playlist,
/// which can be a user or system-generated content.
#[derive(Default, Debug, Serialize, Deserialize, Clone, StructuralConvert, Type)]
#[convert(from(PlaylistCreator))]
#[convert(into(PlaylistCreator))]
pub struct PlaylistCreatorDTO {
    /// The user ID of the playlist creator.
    /// Will be None or zero if the playlist creator is not a known user.
    #[serde(default)]
    pub id: Option<u64>,
}



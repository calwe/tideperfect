use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use specta::Type;
use structural_convert::StructuralConvert;
use strum_macros::{AsRefStr, EnumString};
use tideperfect::services::album::{Album, AlbumSummary, AlbumType, ArtistSummary, FavoriteAlbum};

use crate::dtos::track::{AudioQualityDTO, MediaMetadataDTO};

#[derive(Debug, Serialize, Deserialize, Clone, Type, StructuralConvert)]
#[convert(from(FavoriteAlbum))]
pub struct FavouriteAlbumDTO {
    pub created: String,
    pub item: AlbumDTO,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, Type, StructuralConvert)]
#[convert(from(Album))]
#[serde(rename_all = "camelCase")]
pub struct AlbumDTO {
    /// Unique album identifier
    #[serde_as(as = "DisplayFromStr")]
    pub id: u64,
    /// List of artists who contributed to this album
    #[serde(default = "Default::default")]
    pub artists: Vec<ArtistSummaryDTO>,

    /// Audio quality level available for this album for standard streaming
    ///
    /// Higher quality streams may be available than is indicated here when using MPEG-DASH for playback.
    pub audio_quality: AudioQualityDTO,
    /// Total duration of the album in seconds
    pub duration: u32,
    /// Whether the album contains explicit content
    pub explicit: bool,
    /// Album title
    pub title: String,
    /// Popularity score for the album
    pub popularity: u32,

    /// Additional media metadata and tags
    pub media_metadata: Option<MediaMetadataDTO>,

    /// Album cover image identifier
    ///
    /// Use cover_url() to get the full URL of the cover image.
    pub cover: Option<String>,
    /// Video cover identifier (if available)
    pub video_cover: Option<String>,
    /// Dominant color extracted from the cover art
    pub vibrant_color: Option<String>,
    /// Original release date of the album
    pub release_date: Option<String>,
    /// Date when the album became available for streaming
    pub stream_start_date: Option<String>,

    /// Copyright information
    pub copyright: Option<String>,
    /// Total number of tracks on the album
    pub number_of_tracks: u32,
    /// Number of videos included with the album
    pub number_of_videos: u32,
    /// Number of volumes (for multi-disc albums)
    pub number_of_volumes: u32,
    /// Universal Product Code (UPC) for the album
    pub upc: Option<String>,
    /// Tidal URL for the album
    pub url: String,
    /// Album version or edition
    pub version: Option<String>,

    /// Type of album (ALBUM, EP, Single, etc.)
    #[serde(rename = "type")]
    pub album_type: AlbumTypeDTO,

    /// Whether the album is ready for ad-supported streaming
    pub ad_supported_stream_ready: bool,
    /// Whether streaming is allowed for this album
    pub allow_streaming: bool,
    /// Whether the album is ready for DJ use
    pub dj_ready: bool,
    /// Whether the album requires payment to stream
    pub pay_to_stream: bool,
    /// Whether the album is only available to premium subscribers
    pub premium_streaming_only: bool,
    /// Whether the album supports stem separation
    pub stem_ready: bool,
    /// Whether the album is ready for streaming
    pub stream_ready: bool,

    /// Available audio modes for this album
    pub audio_modes: Vec<String>,
}

/// A simplified representation of an album used in track listings.
///
/// This structure contains only the basic album information
/// and is commonly used in track metadata and search results.
#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, StructuralConvert, Type)]
#[convert(from(AlbumSummary))]
#[serde(rename_all = "camelCase")]
pub struct AlbumSummaryDTO {
    /// Unique album identifier
    #[serde_as(as = "DisplayFromStr")]
    pub id: u64,
    /// Album title
    pub title: String,
    /// Album cover image identifier
    pub cover: Option<String>,
    /// Album release date
    pub release_date: Option<String>,
    /// Dominant color extracted from the cover art
    pub vibrant_color: Option<String>,
    /// Video cover identifier (if available)
    pub video_cover: Option<String>,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, Type, StructuralConvert)]
#[convert(from(ArtistSummary))]
#[serde(rename_all = "camelCase")]
pub struct ArtistSummaryDTO {
    /// Unique artist identifier
    #[serde_as(as = "DisplayFromStr")]
    pub id: u64,
    /// Artist name
    pub name: String,
    /// Artist profile picture identifier
    ///
    /// Use picture_url() to get the full URL of the picture
    pub picture: Option<String>,

    /// Whether the artist has cover art available
    #[serde(default)]
    pub contains_cover: bool,

    /// Popularity score for the artist
    #[serde(default)]
    pub popularity: Option<u32>,

    /// Type/category of the artist
    #[serde(rename = "type")]
    #[serde(default)]
    pub artist_type: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, Type, EnumString, AsRefStr, StructuralConvert)]
#[convert(from(AlbumType))]
#[serde(rename_all = "UPPERCASE")]
#[strum(serialize_all = "UPPERCASE")]
pub enum AlbumTypeDTO {
    /// Standard album release
    #[default]
    ALBUM,
    /// Long play album
    Lp,
    /// Extended play album
    Ep,
    /// Single track release
    Single,
    /// Collection of EPs and singles
    EpsAndSingles,
    /// Compilation album
    Compilations,
}


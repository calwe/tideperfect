use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use specta::Type;
use structural_convert::StructuralConvert;
use strum_macros::{AsRefStr, EnumString};
use tideperfect::services::track::{AudioQuality, MediaMetadata, Track};

use crate::dtos::album::{AlbumSummaryDTO, ArtistSummaryDTO};

/// Represents a track from the Tidal catalog.
///
/// This structure contains all available information about a track,
/// including metadata, audio quality, and associated album/artist data.
#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, StructuralConvert, Type)]
#[convert(from(Track))]
#[serde(rename_all = "camelCase")]
pub struct TrackDTO {
    /// Unique track identifier
    #[serde_as(as = "DisplayFromStr")]
    pub id: u64,
    /// Track number within the album
    pub track_number: u32,
    /// List of artists who contributed to this track
    #[serde(default = "Default::default")]
    pub artists: Vec<ArtistSummaryDTO>,

    /// Album information for this track
    pub album: AlbumSummaryDTO,

    /// Audio quality level available for this album for standard streaming
    ///
    /// Higher quality streams may be available than is indicated here when using MPEG-DASH for playback.
    pub audio_quality: AudioQualityDTO,

    /// Duration of the track in seconds
    pub duration: u32,

    /// Whether the track contains explicit content
    pub explicit: bool,

    /// International Standard Recording Code (ISRC)
    pub isrc: Option<String>,
    /// Popularity score for the track
    pub popularity: u32,
    /// Track title
    pub title: String,

    /// Additional media metadata and tags
    #[serde(rename = "mediaMetadata")]
    pub media_metadata: Option<MediaMetadataDTO>,

    /// Copyright information
    pub copyright: Option<String>,
    /// Tidal URL for the track
    pub url: Option<String>,
    /// Beats per minute (BPM) of the track
    pub bpm: Option<u32>,

    pub upload: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Type, EnumString, AsRefStr, StructuralConvert)]
#[convert(from(AudioQuality))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum AudioQualityDTO {
    /// Low quality (typically 96 kbps AAC)
    Low,
    /// High quality (typically 320 kbps AAC)
    High,
    /// Lossless quality (FLAC, typically 44.1 kHz / 16-bit)
    Lossless,
    /// Hi-Res Lossless quality (FLAC, up to 192 kHz / 24-bit)
    HiResLossless, 
}

#[derive(Debug, Serialize, Deserialize, Clone, Type, StructuralConvert)]
#[convert(from(MediaMetadata))]
#[serde(rename_all = "camelCase")]
pub struct MediaMetadataDTO {
    /// Tags associated with the media
    #[serde(default)]
    pub tags: Vec<String>,
}


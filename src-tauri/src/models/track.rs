use serde::{Deserialize, Serialize};
use specta::Type;

use crate::models::playback::AudioQuality;

// TODO: Rename this to make it clear this specifically refers to what we pass to the frontend
//          - It also should also only contain fields the frontend needs
#[derive(Debug, Serialize, Deserialize, Clone, Type)]
#[serde(rename_all = "camelCase")]
pub struct Track {
    /// Unique track identifier
    #[serde(serialize_with = "to_string", deserialize_with = "from_string")]
    pub id: u64,
    /// Track number within the album
    pub track_number: u32,

    // /// Album information for this track
    // pub album: AlbumSummary,

    // /// Audio quality level available for this album for standard streaming
    // ///
    /// Higher quality streams may be available than is indicated here when using MPEG-DASH for playback.
    pub audio_quality: AudioQuality,

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

    // /// Additional media metadata and tags
    #[serde(rename = "mediaMetadata")]
    pub media_metadata: Option<MediaMetadata>,

    /// Copyright information
    pub copyright: Option<String>,
    /// Tidal URL for the track
    pub url: Option<String>,
    /// Beats per minute (BPM) of the track
    pub bpm: Option<u32>,

    pub upload: Option<bool>,
}

impl From<tidalrs::Track> for Track {
    fn from(value: tidalrs::Track) -> Self {
        Self {
            id: value.id,
            track_number: value.track_number,
            audio_quality: value.audio_quality.into(),
            media_metadata: value.media_metadata.map(|t| t.into()),
            duration: value.duration,
            explicit: value.explicit,
            isrc: value.isrc,
            popularity: value.popularity,
            title: value.title,
            copyright: value.copyright,
            url: value.url,
            bpm: value.bpm,
            upload: value.upload,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct MediaMetadata {
    /// Tags associated with the media
    #[serde(default)]
    pub tags: Vec<String>,
}

impl From<tidalrs::MediaMetadata> for MediaMetadata {
    fn from(value: tidalrs::MediaMetadata) -> Self {
        Self {
            tags: value.tags,
        }
    }
}

fn to_string<S>(value: &u64, serializer: S) -> Result<S::Ok, S::Error>
where
  S: serde::Serializer,
{
  serializer.serialize_str(&value.to_string())
}

fn from_string<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
  D: serde::Deserializer<'de>,
{
  let s = String::deserialize(deserializer)?;
  s.parse().map_err(serde::de::Error::custom)
}


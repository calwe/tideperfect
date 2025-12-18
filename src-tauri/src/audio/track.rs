use std::{sync::Arc, thread};

use base64::prelude::*;
use cpal::Sample;
use dash_mpd::MPD;
use ringbuf::{traits::{Observer, Split}, HeapRb};
use serde::{Deserialize, Serialize};
use specta::Type;
use thiserror::Error;
use tidalrs::TidalClient;
use tracing::{info, instrument, trace};

use crate::{audio::{player::Player, stream::stream_dash_audio}, error::AppError};

pub struct PlayerTrack {
    pub metadata: PlayerTrackMetadata,
    pub buffer: Arc<HeapRb<i32>>,
    pub mpd: Option<MPD>,
    pub url: Option<String>,
}

impl std::fmt::Debug for PlayerTrack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlayerTrack")
            .field("metadata", &self.metadata)
            .field("buffer", &format!("[i32; {}]", &self.buffer.occupied_len() + &self.buffer.vacant_len()))
            .field("mpd", &"MPD {...}")
            .finish()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PlayerTrackMetadata {
    pub id: u64,
    pub sample_rate: u32,
    pub sample_size: u32,
    pub channels: u16,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FlacManifest {
    mime_type: String,
    codecs: String,
    encryption_type: String,
    urls: Vec<String>,
}

#[derive(Debug, Error, Type)]
pub enum PlayerTrackError {
    #[error("Manifest from tidal is currently unsupported: {0}")]
    UnsupportedManifest(String)
}

const BUFFER_SIZE_SECONDS: usize = 500;

impl PlayerTrack {
    #[instrument(skip(client), err)]
    pub async fn fetch(client: &TidalClient, id: u64) -> Result<Self, AppError> {
        info!("Fetching track with id {id}");
        let stream = client.track_dash_playback_info(id, tidalrs::AudioQuality::HiResLossless).await?;

        let manifest = BASE64_STANDARD.decode(stream.manifest)?;
        let manifest = String::from_utf8(manifest)?;

        // check first character of manifest
        //  - if '{', then it is json containing a link to the track audio file
        //  - if '<', then it is a MPEG-DASH manifest, formatted with XML
        match manifest.chars().next().expect("Manifest from Tidal is empty - this is a bug") {
            '{' => {
                let manifest: FlacManifest = serde_json::from_str(&manifest)?;

                // TODO: Proper sample rate
                let sample_rate = 44100;
                let channels = 2;
                let sample_size = 16;

                let metadata = PlayerTrackMetadata {
                    id,
                    sample_rate,
                    sample_size,
                    channels,
                };

                let buffer = Arc::new(HeapRb::<i32>::new(BUFFER_SIZE_SECONDS * sample_rate as usize));

                Ok(Self {
                    metadata,
                    buffer,
                    mpd: None,
                    url: Some(manifest.urls[0].clone()),
                })
            },
            '<' => {
                let mpd = dash_mpd::parse(&manifest)?;

                // TODO: Is this an issue for any song?
                let sample_rate = mpd.periods[0].adaptations[0].representations[0].audioSamplingRate.clone().unwrap().parse().unwrap();
                let channels = 2;
                let sample_size = 24;

                let metadata = PlayerTrackMetadata {
                    id,
                    sample_rate,
                    sample_size,
                    channels,
                };

                let buffer = Arc::new(HeapRb::<i32>::new(BUFFER_SIZE_SECONDS * sample_rate as usize));

                Ok(Self {
                    metadata,
                    buffer,
                    mpd: Some(mpd),
                    url: None,
                })
            },
            _ => Err(PlayerTrackError::UnsupportedManifest(manifest))?
        }
    }
}

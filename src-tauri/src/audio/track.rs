use std::{sync::{atomic::{AtomicBool, AtomicU64, Ordering}, Arc}, time::Duration};

use base64::prelude::*;
use cpal::{traits::{DeviceTrait, StreamTrait}, Device, Sample, SampleFormat, SampleRate, Stream};
use dash_mpd::MPD;
use ringbuf::{traits::{Consumer, Observer, Split}, CachingCons, CachingProd, HeapRb};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Emitter};
use tauri_specta::Event;
use thiserror::Error;
use tokio::{sync::mpsc, task::JoinHandle, time::sleep};
use tidalrs::TidalClient;
use tracing::{error, info, instrument, trace, warn};

use crate::{audio::{player::PlayerCommand, stream::{stream_dash_audio, stream_url}}, error::AppError, models::track::Track};

#[derive(Serialize, Deserialize, Debug, Clone, Type, Event)]
pub struct CurrentTrackEvent(pub Option<Track>);

#[derive(Serialize, Deserialize, Debug, Clone, Type, Event)]
pub struct PlaybackProgressEvent(pub u32);

pub struct PlayerTrack {
    pub metadata: PlayerTrackMetadata,
    pub track: Track,
    pub buffer: Arc<HeapRb<i32>>,
    pub stream: Option<Stream>,
    pub mpd: Option<MPD>,
    pub url: Option<String>,
    pub samples_played: Arc<AtomicU64>,
    pub progress_handle: Option<JoinHandle<()>>,
}

impl std::fmt::Debug for PlayerTrack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlayerTrack")
            .field("metadata", &self.metadata)
            .field("buffer", &format!("[i32; {}]", self.buffer.occupied_len() + self.buffer.vacant_len()))
            .field("mpd", &"MPD {...}")
            .finish()
    }
}

#[derive(Debug, Clone, Copy, Type)]
pub struct PlayerTrackMetadata {
    pub id: u64,
    pub sample_rate: u32,
    pub sample_size: u32,
    pub channels: u16,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FlacManifest {
    _mime_type: String,
    _codecs: String,
    _encryption_type: String,
    urls: Vec<String>,
}

#[derive(Debug, Error, Type)]
pub enum PlayerTrackError {
    #[error("Manifest from tidal is currently unsupported: {0}")]
    UnsupportedManifest(String),
    #[error("Cannot find a config suitable for playing the track: {0:?}")]
    UnsupportedConfig(PlayerTrackMetadata),
    #[error("SupportedStreamConfigsError: {0}")]
    SupportedStreamConfigs(String),
    #[error("Track sample_size ({0}) is unsupported by the player")]
    UnsupportedSampleSize(u32),
}

const BUFFER_SIZE_SECONDS: usize = 5;

impl PlayerTrack {
    #[instrument(skip(client), err)]
    pub async fn fetch(client: &TidalClient, id: u64) -> Result<Self, AppError> {
        info!("Fetching track with id {id}");
        let stream = client.track_dash_playback_info(id, tidalrs::AudioQuality::HiResLossless).await?;
        let track: Track = client.track(id).await?.into();

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
                    track,
                    buffer,
                    samples_played: Arc::new(AtomicU64::new(0)),
                    stream: None,
                    mpd: None,
                    url: Some(manifest.urls[0].clone()),
                    progress_handle: None,
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
                    samples_played: Arc::new(AtomicU64::new(0)),
                    track,
                    stream: None,
                    mpd: Some(mpd),
                    url: None,
                    progress_handle: None,
                })
            },
            _ => Err(PlayerTrackError::UnsupportedManifest(manifest))?
        }
    }

    pub fn start_playback(&mut self, device: &Device, player_tx: mpsc::Sender<PlayerCommand>, paused: Arc<AtomicBool>, app_handle: AppHandle) -> Result <(), PlayerTrackError> {
        info!("Playing track (ID #{})", self.metadata.id);

        self.stream = None;

        let buffer = self.buffer.clone();
        let (producer, mut consumer) = buffer.split();

        // Track when streaming is complete
        let streaming_done = Arc::new(AtomicBool::new(false));
        let track_finished_sent = Arc::new(AtomicBool::new(false));

        // begin filling buffer
        Self::stream(producer, self.mpd.clone(), self.url.clone(), streaming_done.clone());

        let metadata = self.metadata;
        let supported_configs = device.supported_output_configs().map_err(|e| PlayerTrackError::SupportedStreamConfigs(e.to_string()))?;
        let supported_config = supported_configs
            .filter(|c| c.channels() == metadata.channels)
            .find(|c| c.sample_format() == Self::sample_size_to_format(metadata.sample_size))
            .ok_or(PlayerTrackError::UnsupportedConfig(metadata))?
            .try_with_sample_rate(SampleRate(metadata.sample_rate))
            .ok_or(PlayerTrackError::UnsupportedConfig(metadata))?;
        trace!("Using supported config: {supported_config:?}");

        let samples_played = self.samples_played.clone();
        let err_fn = |err| error!("an error occurred on the output audio stream: {}", err);
        let stream = match metadata.sample_size {
            16 => {
                let streaming_done_clone = streaming_done.clone();
                let track_finished_sent_clone = track_finished_sent.clone();
                let player_tx_clone = player_tx.clone();

                device.build_output_stream(
                    &supported_config.config(),
                    move |data, _| {
                        let buffer_empty = Self::write_audio_data_16_bit(data, &mut consumer, paused.clone(), samples_played.clone());

                        // If streaming is done AND buffer is empty AND we haven't sent the signal yet
                        if buffer_empty &&
                           streaming_done_clone.load(Ordering::Relaxed) &&
                           !track_finished_sent_clone.swap(true, Ordering::Relaxed) {
                            info!("Track playback complete (buffer empty and streaming done)");
                            let _ = player_tx_clone.try_send(PlayerCommand::Skip);
                        }
                    },
                    err_fn,
                    None
                )
            }
            24 => {
                let streaming_done_clone = streaming_done.clone();
                let track_finished_sent_clone = track_finished_sent.clone();
                let player_tx_clone = player_tx.clone();

                device.build_output_stream(
                    &supported_config.config(),
                    move |data, _| {
                        let buffer_empty = Self::write_audio_data_24_bit(data, &mut consumer, paused.clone(), samples_played.clone());

                        // If streaming is done AND buffer is empty AND we haven't sent the signal yet
                        if buffer_empty &&
                           streaming_done_clone.load(Ordering::Relaxed) &&
                           !track_finished_sent_clone.swap(true, Ordering::Relaxed) {
                            info!("Track playback complete (buffer empty and streaming done)");
                            let _ = player_tx_clone.try_send(PlayerCommand::Skip);
                        }
                    },
                    err_fn,
                    None
                )
            }
            _ => return Err(PlayerTrackError::UnsupportedSampleSize(metadata.sample_size))
        }
        .unwrap();

        trace!("Made stream");

        stream.play().unwrap();

        trace!("Playing stream");

        let samples_played = self.samples_played.clone();

        self.progress_handle = Some(tokio::spawn(async move {
            loop {
                let samples_played = samples_played.load(Ordering::Relaxed);
                let progress = samples_played / metadata.channels as u64 / metadata.sample_rate as u64;
                let _ = PlaybackProgressEvent(progress as u32).emit(&app_handle);
                sleep(Duration::from_millis(100)).await;
            }
        }));

        self.stream = Some(stream);

        Ok(())
    }

    pub fn stop_track(&mut self) {
        let buffer = Arc::new(HeapRb::<i32>::new(BUFFER_SIZE_SECONDS * self.metadata.sample_rate as usize));

        self.buffer = buffer;

        if let Some(handle) = &self.progress_handle {
            handle.abort();
            self.progress_handle = None;
        }

        self.stream = None;
    }

    #[instrument(skip(producer, streaming_done))]
    fn stream(
        producer: CachingProd<Arc<HeapRb<i32>>>,
        mpd: Option<MPD>,
        url: Option<String>,
        streaming_done: Arc<AtomicBool>
    ) -> JoinHandle<()> {
        if let Some(mpd) = mpd {
            tokio::spawn(async move {
                if let Err(error) = stream_dash_audio(producer, mpd).await {
                    error!("Stream Error: {error}");
                }
                streaming_done.store(true, Ordering::Relaxed);
                info!("Streaming complete (buffer filled)");
            })
        } else if let Some(url) = url {
            tokio::spawn(async move {
                if let Err(error) = stream_url(producer, url).await {
                    error!("Stream Error: {error}");
                }
                streaming_done.store(true, Ordering::Relaxed);
                info!("Streaming complete (buffer filled)");
            })
        } else {
            unreachable!("Stream function did not recieve a URl or MPD to stream from. This is a bug.");
        }
    }

    #[instrument(skip(output, consumer))]
    fn write_audio_data_24_bit(
        output: &mut [i32],
        consumer: &mut CachingCons<Arc<HeapRb<i32>>>,
        paused: Arc<AtomicBool>,
        samples_played: Arc<AtomicU64>,
    ) -> bool {
        let mut i = 0;
        let mut buffer_was_empty = false;

        while i < output.len() {
            if paused.load(Ordering::Relaxed) {
                output[i] = i32::EQUILIBRIUM;
                i += 1;
                continue;
            }

            if let Some(sample) = consumer.try_pop() {
                output[i] = sample;
                samples_played.fetch_add(1, Ordering::Relaxed);
                i += 1;
            } else {
                buffer_was_empty = true;
                for sample in &mut output[i..] {
                    *sample = i32::EQUILIBRIUM;
                }
                break;
            }
        }

        buffer_was_empty
    }

    #[instrument(skip(output, consumer))]
    fn write_audio_data_16_bit(
        output: &mut [i16],
        consumer: &mut CachingCons<Arc<HeapRb<i32>>>,
        paused: Arc<AtomicBool>,
        samples_played: Arc<AtomicU64>,
    ) -> bool {
        let mut i = 0;
        let mut buffer_was_empty = false;

        while i < output.len() {
            if paused.load(Ordering::Relaxed) {
                output[i] = i16::EQUILIBRIUM;
                i += 1;
                continue;
            }

            if let Some(sample) = consumer.try_pop() {
                output[i] = sample.to_sample();
                samples_played.fetch_add(1, Ordering::Relaxed);
                i += 1;
            } else {
                warn!("Buffer empty!");
                buffer_was_empty = true;
                for sample in &mut output[i..] {
                    *sample = i16::EQUILIBRIUM;
                }
                break;
            }
        }

        buffer_was_empty
    }
    
    // TODO: I need to find out more about what sample format to use... does it really matter?
    fn sample_size_to_format(sample_size: u32) -> SampleFormat {
        match sample_size {
            16 => SampleFormat::I16,
            24 => SampleFormat::I32,
            _ => SampleFormat::I32,
        }
    }
}

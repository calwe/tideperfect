use std::{string::FromUtf8Error, sync::{atomic::{AtomicBool, AtomicU64, Ordering}, Arc}, time::Duration};

use base64::prelude::*;
use cpal::{traits::{DeviceTrait, StreamTrait}, Device, Sample, SampleFormat, SampleRate, Stream};
use dash_mpd::MPD;
use ringbuf::{traits::{Consumer, Observer, Split}, CachingCons, CachingProd, HeapRb};
use serde::Deserialize;
use snafu::{ResultExt, Snafu};
use tokio::{sync::{broadcast, mpsc}, task::JoinHandle, time::sleep};
use tidalrs::{TidalClient, Track as TidalTrack, TrackDashPlaybackInfo};
use tracing::{error, info, instrument, trace, warn};

use crate::{audio::{player::{PlayerCommand, PlayerEvent}, stream::{stream_dash_audio, stream_url}}, Event};

pub struct Track {
    pub metadata: TrackMetadata,
    pub track: TidalTrack,
    pub buffer: Arc<HeapRb<i32>>,
    pub stream: Option<Stream>,
    pub mpd: Option<MPD>,
    pub url: Option<String>,
    pub samples_played: Arc<AtomicU64>,
    pub progress_handle: Option<JoinHandle<()>>,
}

impl std::fmt::Debug for Track {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlayerTrack")
            .field("metadata", &self.metadata)
            .field("buffer", &format!("[i32; {}]", self.buffer.occupied_len() + self.buffer.vacant_len()))
            .field("mpd", &"MPD {...}")
            .finish()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TrackMetadata {
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

const BUFFER_SIZE_SECONDS: usize = 5;

impl Track {

    #[instrument(skip(client), err)]
    pub async fn fetch(client: &TidalClient, id: u64) -> Result<Self, TrackError> {
        info!("Fetching track with id {id}");
        let track = client.track(id).await.context(TidalSnafu)?;
        Self::fetch_from_track(client, &track).await
    }

    #[instrument(skip_all, err)]
    pub async fn fetch_from_track(client: &TidalClient, track: &TidalTrack) -> Result<Self, TrackError> {
        info!("Fetching track from supplied tidal track");
        let stream = client.track_dash_playback_info(track.id, tidalrs::AudioQuality::HiResLossless).await
            .context(TidalSnafu)?;

        Self::parse_manifest(stream, track)
    }

    fn parse_manifest(stream: TrackDashPlaybackInfo, track: &TidalTrack) -> Result<Self, TrackError> {
        let manifest = BASE64_STANDARD.decode(stream.manifest).context(Base64Snafu)?;
        let manifest = String::from_utf8(manifest).context(UTF8Snafu)?;

        // check first character of manifest
        //  - if '{', then it is json containing a link to the track audio file
        //  - if '<', then it is a MPEG-DASH manifest, formatted with XML
        match manifest.chars().next().expect("Manifest from Tidal is empty - this is a bug") {
            '{' => {
                let manifest: FlacManifest = serde_json::from_str(&manifest).context(SerdeSnafu)?;

                // TODO: Proper sample rate
                let sample_rate = 44100;
                let channels = 2;
                let sample_size = 16;

                let metadata = TrackMetadata {
                    id: track.id,
                    sample_rate,
                    sample_size,
                    channels,
                };

                let buffer = Arc::new(HeapRb::<i32>::new(BUFFER_SIZE_SECONDS * sample_rate as usize));

                Ok(Self {
                    metadata,
                    track: track.clone(),
                    buffer,
                    samples_played: Arc::new(AtomicU64::new(0)),
                    stream: None,
                    mpd: None,
                    url: Some(manifest.urls[0].clone()),
                    progress_handle: None,
                })
            },
            '<' => {
                let mpd = dash_mpd::parse(&manifest).context(MPDSnafu)?;

                // TODO: Is this an issue for any song?
                let sample_rate = mpd.periods[0].adaptations[0].representations[0].audioSamplingRate.clone().unwrap().parse().unwrap();
                let channels = 2;
                let sample_size = 24;

                let metadata = TrackMetadata {
                    id: track.id,
                    sample_rate,
                    sample_size,
                    channels,
                };

                let buffer = Arc::new(HeapRb::<i32>::new(BUFFER_SIZE_SECONDS * sample_rate as usize));

                Ok(Self {
                    metadata,
                    buffer,
                    samples_played: Arc::new(AtomicU64::new(0)),
                    track: track.clone(),
                    stream: None,
                    mpd: Some(mpd),
                    url: None,
                    progress_handle: None,
                })
            },
            _ => Err(TrackError::UnsupportedManifest { manifest })?
        }
    }

    pub fn start_playback(&mut self, device: &Device, event_emitter: broadcast::Sender<Event>, player_tx: mpsc::Sender<PlayerCommand>, paused: Arc<AtomicBool>) -> Result <(), TrackError> {
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
        let supported_configs = device.supported_output_configs().context(SupportedStreamConfigsSnafu)?;
        let supported_config = supported_configs
            .filter(|c| c.channels() == metadata.channels)
            .find(|c| c.sample_format() == Self::sample_size_to_format(metadata.sample_size))
            .ok_or(TrackError::UnsupportedConfig { metadata })?
            .try_with_sample_rate(SampleRate(metadata.sample_rate))
            .ok_or(TrackError::UnsupportedConfig { metadata })?;
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
            _ => return Err(TrackError::UnsupportedSampleSize { sample_size: metadata.sample_size })
        }
        .unwrap();

        trace!("Made stream");

        stream.play().unwrap();

        trace!("Playing stream");

        let samples_played = self.samples_played.clone();

        self.progress_handle = Some(tokio::spawn(async move {
            loop {
                let samples_played = samples_played.load(Ordering::SeqCst);
                let progress = samples_played / metadata.channels as u64 / metadata.sample_rate as u64;
                event_emitter.send(Event::PlayerEvent(PlayerEvent::UpdatedTrackProgress(progress as u32)));
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
            self.samples_played.store(0, Ordering::SeqCst);
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
                streaming_done.store(true, Ordering::SeqCst);
                info!("Streaming complete (buffer filled)");
            })
        } else if let Some(url) = url {
            tokio::spawn(async move {
                if let Err(error) = stream_url(producer, url).await {
                    error!("Stream Error: {error}");
                }
                streaming_done.store(true, Ordering::SeqCst);
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
            if paused.load(Ordering::SeqCst) {
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

#[derive(Debug, Snafu)]
pub enum TrackError {
    #[snafu(display("Error from TidalAPI"))]
    Tidal {
        source: tidalrs::Error,
    },
    #[snafu(display("Failed to convert manifest from base64"))]
    Base64 {
        source: base64::DecodeError,
    },
    #[snafu(display("Failed to convert base64 to string"))]
    UTF8 {
        source: FromUtf8Error,
    },
    #[snafu(display("Failed to parse URL manifest"))]
    Serde {
        source: serde_json::Error,
    },
    #[snafu(display("Failed to parse MPD manifest"))]
    MPD {
        source: dash_mpd::DashMpdError,
    },
    #[snafu(display("Error getting supported output configs"))]
    SupportedStreamConfigs {
        source: cpal::SupportedStreamConfigsError,
    },
    #[snafu(display("Device does not support playing track: {metadata:?}"))]
    UnsupportedConfig {
        metadata: TrackMetadata,
    },
    #[snafu(display("TidePerfect cannot play track with sample size: {sample_size}"))]
    UnsupportedSampleSize {
        sample_size: u32,
    },
    #[snafu(display("Unsupported manifest type: {manifest}"))]
    UnsupportedManifest {
        manifest: String,
    },
}

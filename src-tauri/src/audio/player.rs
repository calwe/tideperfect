use std::sync::Arc;
use tokio::task::JoinHandle;
use cpal::{default_host, traits::{DeviceTrait, HostTrait, StreamTrait}, Device, Host, Sample, SampleFormat, SampleRate, Stream, SupportedStreamConfigsError};
use dash_mpd::MPD;
use ringbuf::{traits::{Consumer, Split}, CachingCons, CachingProd, HeapRb};
use thiserror::Error;
use tracing::{error, info, instrument, trace, warn};

use crate::audio::{stream::{stream_dash_audio, stream_url}, track::{PlayerTrack, PlayerTrackMetadata}};

pub struct Player {
    host: Host,
    device: Device,
    stream: Option<Stream>,
    stream_handle: Option<JoinHandle<()>>,
}

#[derive(Error, Debug)]
pub enum PlayerError {
    #[error("No default device for host")]
    NoDefaultDevice,
    #[error("Could not get any output devices")]
    NoDevices,
    #[error("Could not retrieve a devices name: {0}")]
    DeviceName(String),
    #[error("Could not retrieve any configs for the device")]
    NoSupportedConfigs,
    #[error("Cannot find a config suitable for playing the track: {0:?}")]
    UnsupportedConfig(PlayerTrackMetadata),
    #[error(transparent)]
    SupportedStreamConfigs(#[from] SupportedStreamConfigsError),
    #[error("Track sample_size ({0}) is unsupported by the player")]
    UnsupportedSampleSize(u32),
}

impl Player {
    #[instrument(err)]
    pub fn init_default_output() -> Result<Self, PlayerError> {
        let host = default_host();
        let device = host.default_output_device().ok_or(PlayerError::NoDefaultDevice)?;
        trace!("Using device with name: {:?}", device.name());

        Ok(Self {
            host,
            device,
            stream: None,
            stream_handle: None,
        })
    }

    #[instrument(skip(self), err)]
    pub fn devices(&self) -> Result<Vec<String>, PlayerError> {
        let devices = self.host.output_devices().map_err(|_| PlayerError::NoDevices)?;
        Ok(devices.filter_map(|device| device.name().ok()).collect())
    }

    #[instrument(skip(self), err)]
    pub fn set_device(&mut self, device: &str) -> Result<(), PlayerError> {
        let mut devices = self.host.output_devices().map_err(|_| PlayerError::NoDevices)?;
        let device = devices.find(|d| d.name() == Ok(device.to_owned())).ok_or(PlayerError::NoDefaultDevice)?;
        self.device = device;

        Ok(())
    }

    // TODO: Queue system
    #[instrument(skip(self), fields(device = %self.device.name().unwrap_or("invalid_name".to_owned())), err)]
    pub fn play_track(&mut self, track: PlayerTrack) -> Result <(), PlayerError> {
        info!("Playing track (ID #{})", track.metadata.id);

        self.stream = None;
        
        if let Some(handle) = &self.stream_handle {
            handle.abort();
        }

        let (producer, mut consumer) = track.buffer.split();

        // begin filling buffer
        self.stream_handle = Some(Self::stream(producer, track.mpd, track.url));

        let metadata = track.metadata;
        let supported_configs = self.device.supported_output_configs()?;
        let supported_config = supported_configs
            .filter(|c| c.channels() == metadata.channels)
            .find(|c| c.sample_format() == Self::sample_size_to_format(metadata.sample_size))
            .ok_or(PlayerError::UnsupportedConfig(metadata))?
            .try_with_sample_rate(SampleRate(metadata.sample_rate))
            .ok_or(PlayerError::UnsupportedConfig(metadata))?;
        trace!("Using supported config: {supported_config:?}");

        let err_fn = |err| error!("an error occurred on the output audio stream: {}", err);
        let stream = match metadata.sample_size {
            16 => {
                self.device.build_output_stream(
                    &supported_config.config(),
                    move |data, _| {
                        Self::write_audio_data_16_bit(data, &mut consumer);
                    },
                    err_fn, 
                    None
                )
            }
            24 => {
                self.device.build_output_stream(
                    &supported_config.config(),
                    move |data, _| {
                        Self::write_audio_data_24_bit(data, &mut consumer);
                    },
                    err_fn, 
                    None
                )
            }
            _ => return Err(PlayerError::UnsupportedSampleSize(metadata.sample_size))
        }
        .unwrap();

        trace!("Made stream");

        stream.play().unwrap();

        trace!("Playing stream");

        self.stream = Some(stream);

        Ok(())
    }

    #[instrument(skip(self), fields(device = %self.device.name().unwrap_or("invalid_name".to_owned())))]
    pub fn stop_track(&mut self) {
        self.stream = None;

        if let Some(handle) = &self.stream_handle {
            handle.abort();
        }
        self.stream_handle = None;
    }

    #[instrument(skip(producer))]
    fn stream(producer: CachingProd<Arc<HeapRb<i32>>>, mpd: Option<MPD>, url: Option<String>) -> JoinHandle<()> {
        if let Some(mpd) = mpd {
            tokio::spawn(async move {
                if let Err(error) = stream_dash_audio(producer, mpd).await {
                    error!("Stream Error: {error}");
                }
            })
        } else if let Some(url) = url {
            tokio::spawn(async move {
                if let Err(error) = stream_url(producer, url).await {
                    error!("Stream Error: {error}");
                }
            })
        } else {
            unreachable!("Stream function did not recieve a URl or MPD to stream from. This is a bug.");
        }
    }

    #[instrument(skip(output, consumer))]
    fn write_audio_data_24_bit(
        output: &mut [i32],
        consumer: &mut CachingCons<Arc<HeapRb<i32>>>,
    ) {
        let mut i = 0;

        while i < output.len() {
            if let Some(sample) = consumer.try_pop() {
                    output[i] = sample;
                i += 1;
            } else {
                //warn!("No samples in buffer! Playing silence.");
                for sample in &mut output[i..] {
                    *sample = i32::EQUILIBRIUM;
                }
                break;
            }
        }
    }

    #[instrument(skip(output, consumer))]
    fn write_audio_data_16_bit(
        output: &mut [i16],
        consumer: &mut CachingCons<Arc<HeapRb<i32>>>,
    ) {
        let mut i = 0;

        while i < output.len() {
            if let Some(sample) = consumer.try_pop() {
                output[i] = sample.to_sample();
                i += 1;
            } else {
                //warn!("No samples in buffer! Playing silence.");
                for sample in &mut output[i..] {
                    *sample = i16::EQUILIBRIUM;
                }
                break;
            }
        }
    }
    
    // fn write_silence<T: Sample>(data: &mut [T], _: &cpal::OutputCallbackInfo) {
    //     for sample in data.iter_mut() {
    //         *sample = Sample::EQUILIBRIUM;
    //     }
    // }

    // TODO: I need to find out more about what sample format to use... does it really matter?
    fn sample_size_to_format(sample_size: u32) -> SampleFormat {
        match sample_size {
            16 => SampleFormat::I16,
            24 => SampleFormat::I32,
            _ => SampleFormat::I32,
        }
    }
}

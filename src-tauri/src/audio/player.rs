use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{App, AppHandle};
use tauri_specta::Event;
use tokio::{sync::{mpsc, Mutex}};
use cpal::{default_host, traits::{DeviceTrait, HostTrait}, Device, Host};
use thiserror::Error;
use tracing::{error, info, instrument, trace};

use crate::{audio::track::{CurrentTrackEvent, PlayerTrack}, models::queue::Queue};

pub struct Player {
    control_tx: mpsc::Sender<PlayerCommand>,
    pub queue: Arc<Mutex<Queue>>,
}

#[derive(Debug, Clone, Type, Event, Serialize, Deserialize)]
pub struct PlaybackStateEvent(bool);

pub enum PlayerCommand {
    Play,
    Pause,
    Resume,
    TrackFinished,
    Skip,
    SwitchDevice(String),
}

#[derive(Error, Debug, Type)]
pub enum PlayerError {
    #[error("No default device for host")]
    NoDefaultDevice,
    #[error("Background thread died")]
    BackgroundThreadDied,
}

impl Player {
    #[instrument(skip_all, err)]
    pub fn init_default_output(app_handle: AppHandle) -> Result<Self, PlayerError> {
        let host = default_host();
        let device = host.default_output_device().ok_or(PlayerError::NoDefaultDevice)?;
        trace!("Using device with name: {:?}", device.name());

        let queue = Arc::new(Mutex::new(Queue::new(app_handle.clone())));
        let (control_tx, control_rx) = mpsc::channel(32);

        tokio::spawn(player_loop(app_handle, control_rx, control_tx.clone(), host, device.clone(), queue.clone()));

        Ok(Self {
            control_tx,
            queue,
        })
    }

    pub async fn play_queue(&self) -> Result<(), PlayerError> {
        self.control_tx.send(PlayerCommand::Play).await
            .map_err(|_| PlayerError::BackgroundThreadDied)?;
        Ok(())
    }

    pub async fn pause(&self) ->  Result<(), PlayerError> {
        self.control_tx.send(PlayerCommand::Pause).await
            .map_err(|_| PlayerError::BackgroundThreadDied)?;
        Ok(())
    }

    pub async fn resume(&self) ->  Result<(), PlayerError> {
        self.control_tx.send(PlayerCommand::Resume).await
            .map_err(|_| PlayerError::BackgroundThreadDied)?;
        Ok(())
    }

    pub async fn skip(&self) -> Result<(), PlayerError> {
        self.control_tx.send(PlayerCommand::Skip).await
            .map_err(|_| PlayerError::BackgroundThreadDied)?;
        Ok(())
    }
}

pub async fn player_loop(
    app_handle: AppHandle,
    mut command_rx: mpsc::Receiver<PlayerCommand>,
    command_tx: mpsc::Sender<PlayerCommand>,
    host: Host,
    default_device: Device,
    queue: Arc<Mutex<Queue>>,
) {
    let mut current_track: Option<PlayerTrack> = None;
    let mut is_playing = false;
    let device = default_device;
    let paused = Arc::new(AtomicBool::new(false));

    loop {
        tokio::select! {
            Some(cmd) = command_rx.recv() => {
                match cmd {
                    PlayerCommand::Play => {
                        info!("Got play command. Playing queue");
                        is_playing = true;

                        if current_track.is_none() {
                            if let Some(mut track) = queue.lock().await.deque() {
                                info!("Starting first track in queue");
                                CurrentTrackEvent(Some(track.track.clone())).emit(&app_handle).unwrap();
                                PlaybackStateEvent(true).emit(&app_handle).unwrap();
                                if let Err(e) = track.start_playback(&device, command_tx.clone(), paused.clone()) {
                                    error!("Failed to start track: {}", e);
                                } else {
                                    current_track = Some(track);
                                }
                            }
                        }
                    }
                    PlayerCommand::Pause => {
                        info!("Pausing current track");
                        paused.store(true, Ordering::Relaxed);
                        PlaybackStateEvent(false).emit(&app_handle).unwrap();
                    }
                    PlayerCommand::Resume => {
                        info!("Resuming current track");
                        paused.store(false, Ordering::Relaxed);
                        PlaybackStateEvent(true).emit(&app_handle).unwrap();
                    }
                    PlayerCommand::TrackFinished => {
                        info!("Track finished, starting next track");

                        if let Some(mut track) = current_track.take() {
                            track.stop_track();
                        }

                        if is_playing {
                            if let Some(mut track) = queue.lock().await.deque() {
                                CurrentTrackEvent(Some(track.track.clone())).emit(&app_handle).unwrap();
                                if let Err(e) = track.start_playback(&device, command_tx.clone(), paused.clone()) {
                                    error!("Failed to start next track: {}", e);
                                } else {
                                    current_track = Some(track);
                                }
                            } else {
                                info!("Queue is empty, stopping playback");
                                CurrentTrackEvent(None).emit(&app_handle).unwrap();
                                is_playing = false;
                            }
                        }
                    }
                    PlayerCommand::Skip => {
                        // TODO: This should not deque from queue, but instead just move on?
                        info!("Skipping current track");
                        if let Some(mut track) = current_track.take() {
                            track.stop_track();
                        }

                        if is_playing {
                            if let Some(mut track) = queue.lock().await.deque() {
                                CurrentTrackEvent(Some(track.track.clone())).emit(&app_handle).unwrap();
                                if let Err(e) = track.start_playback(&device, command_tx.clone(), paused.clone()) {
                                    error!("Failed to start next track: {}", e);
                                } else {
                                    current_track = Some(track);
                                }
                            } else {
                                info!("Queue is empty, stopping playback");
                                CurrentTrackEvent(None).emit(&app_handle).unwrap();
                                is_playing = false;
                            }
                        }
                    }
                    PlayerCommand::SwitchDevice(device) => {}
                }
            }
        }
    }
}

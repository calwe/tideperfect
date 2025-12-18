use std::sync::Arc;
use specta::Type;
use tokio::{sync::{mpsc, Mutex}};
use cpal::{default_host, traits::{DeviceTrait, HostTrait}, Device, Host};
use thiserror::Error;
use tracing::{error, info, instrument, trace};

use crate::{audio::{track::{PlayerTrack}}, models::queue::Queue};

pub struct Player {
    control_tx: mpsc::Sender<PlayerCommand>,
    pub queue: Arc<Mutex<Queue>>,
}

pub enum PlayerCommand {
    Play,
    TrackFinished,
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
    #[instrument(err)]
    pub fn init_default_output() -> Result<Self, PlayerError> {
        let host = default_host();
        let device = host.default_output_device().ok_or(PlayerError::NoDefaultDevice)?;
        trace!("Using device with name: {:?}", device.name());

        let queue = Arc::new(Mutex::new(Queue::new()));
        let (control_tx, control_rx) = mpsc::channel(32);

        tokio::spawn(player_loop(control_rx, control_tx.clone(), host, device.clone(), queue.clone()));

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
}

pub async fn player_loop(
    mut command_rx: mpsc::Receiver<PlayerCommand>,
    command_tx: mpsc::Sender<PlayerCommand>,
    host: Host,
    default_device: Device,
    queue: Arc<Mutex<Queue>>,
) {
    let mut current_track: Option<PlayerTrack> = None;
    let mut is_playing = false;
    let mut device = default_device;

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
                                if let Err(e) = track.start_playback(&device, command_tx.clone()) {
                                    error!("Failed to start track: {}", e);
                                } else {
                                    current_track = Some(track);
                                }
                            }
                        }
                    }
                    PlayerCommand::TrackFinished => {
                        info!("Track finished, starting next track");

                        if let Some(mut track) = current_track.take() {
                            track.stop_track();
                        }

                        if is_playing {
                            if let Some(mut track) = queue.lock().await.deque() {
                                if let Err(e) = track.start_playback(&device, command_tx.clone()) {
                                    error!("Failed to start next track: {}", e);
                                } else {
                                    current_track = Some(track);
                                }
                            } else {
                                info!("Queue is empty, stopping playback");
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

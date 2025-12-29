use std::{str::FromStr, sync::{atomic::{AtomicBool, Ordering}, Arc}};

use cpal::{traits::{DeviceTrait, HostTrait}, Device, Devices};
use serde::de;
use snafu::Report;
use strum_macros::EnumDiscriminants;
use tokio::sync::{broadcast, mpsc, oneshot, Mutex};
use tracing::{error, info, trace};
use cpal::{Host, DeviceId};

use crate::{audio::{queue::Queue, track::Track}, Event};

pub enum PlayerCommand {
    Play,
    Pause,
    Skip,
    Previous,
    SwitchDevice(String),
    GetDevices(oneshot::Sender<Vec<CommandDevice>>)
}

pub struct CommandDevice {
    pub name: String,
    pub id: String,
}

#[derive(Debug, Clone, EnumDiscriminants)]
pub enum PlayerEvent {
    UpdatedCurrentTrack(Option<Box<tidalrs::Track>>),
    UpdatedPauseState(bool),
    UpdatedTrackProgress(u32),
}

// TODO: Should we be passing references for Senders/Recievers? How about other types?
pub async fn player_loop(
    mut command_rx: mpsc::Receiver<PlayerCommand>,
    command_tx: mpsc::Sender<PlayerCommand>,
    event_emitter: broadcast::Sender<Event>,
    host: Host,
    default_device: Device,
    queue: Arc<Mutex<Queue>>,
    played: Arc<Mutex<Vec<Track>>>,
) {
    let mut current_track: Option<Track> = None;
    let mut device = default_device;
    let paused = Arc::new(AtomicBool::new(true));

    // TODO: We need to properly handle results in this thread
    loop {
        tokio::select! {
            Some(cmd) = command_rx.recv() => {
                match cmd {
                    PlayerCommand::Play => {
                        info!("Got play command. Playing queue");
                        paused.store(false, Ordering::SeqCst);
                        event_emitter.send(Event::PlayerEvent(PlayerEvent::UpdatedPauseState(false)));

                        if current_track.is_none() {
                            if let Ok(Some(mut track)) = queue.lock().await.deque() {
                                info!("Starting first track in queue");

                                if let Err(e) = track.start_playback(&device, event_emitter.clone(), command_tx.clone(), paused.clone()) {
                                    error!("Failed to start track: {}", Report::from_error(e));
                                } else {
                                    event_emitter.send(Event::PlayerEvent(PlayerEvent::UpdatedCurrentTrack(Some(Box::new(track.track.clone())))));
                                    current_track = Some(track);
                                }
                            }
                        }
                    }
                    PlayerCommand::Pause => {
                        info!("Pausing current track");
                        paused.store(true, Ordering::SeqCst);
                        event_emitter.send(Event::PlayerEvent(PlayerEvent::UpdatedPauseState(true)));
                    }
                    PlayerCommand::Skip => {
                        info!("Skipping current track");

                        if let Some(mut track) = current_track.take() {
                            track.stop_track();
                            played.lock().await.push(track);
                        }

                        if let Ok(Some(mut track)) = queue.lock().await.deque() {
                            if let Err(e) = track.start_playback(&device, event_emitter.clone(), command_tx.clone(), paused.clone()) {
                                error!("Failed to start next track: {}", e);
                            } else {
                                event_emitter.send(Event::PlayerEvent(PlayerEvent::UpdatedCurrentTrack(Some(Box::new(track.track.clone())))));
                                event_emitter.send(Event::PlayerEvent(PlayerEvent::UpdatedPauseState(false)));
                                paused.store(false, Ordering::SeqCst);
                                current_track = Some(track);
                            }
                        } else {
                            info!("Queue is empty, stopping playback");
                            paused.store(true, Ordering::SeqCst);

                            event_emitter.send(Event::PlayerEvent(PlayerEvent::UpdatedCurrentTrack(None)));
                            event_emitter.send(Event::PlayerEvent(PlayerEvent::UpdatedPauseState(true)));
                        }
                    }
                    PlayerCommand::Previous => {
                        info!("Playing previous track");
                        if let Some(mut track) = current_track.take() {
                            track.stop_track();
                        }

                        if let Some(mut track) = played.lock().await.pop() {
                            if let Err(e) = track.start_playback(&device, event_emitter.clone(), command_tx.clone(), paused.clone()) {
                                error!("Failed to start next track: {}", e);
                            } else {
                                event_emitter.send(Event::PlayerEvent(PlayerEvent::UpdatedCurrentTrack(Some(Box::new(track.track.clone())))));
                                event_emitter.send(Event::PlayerEvent(PlayerEvent::UpdatedPauseState(false)));
                                paused.store(false, Ordering::SeqCst);
                                current_track = Some(track);
                            }
                        } else {
                            info!("No previous track, stopping playback");
                            paused.store(true, Ordering::SeqCst);

                            event_emitter.send(Event::PlayerEvent(PlayerEvent::UpdatedCurrentTrack(None)));
                            event_emitter.send(Event::PlayerEvent(PlayerEvent::UpdatedPauseState(true)));
                        }
                    }
                    PlayerCommand::SwitchDevice(new_device) => {
                        info!("Switching to device {new_device}");
                        let device_id = DeviceId::from_str(&new_device).unwrap();
                        device = host.device_by_id(&device_id).unwrap();
                    }
                    PlayerCommand::GetDevices(sender) => {
                        trace!("Getting devices");
                        let devices = host.devices().unwrap();
                        let devices = devices.map(|device| CommandDevice {
                            name: device.description().unwrap().name().to_owned(),
                            id: device.id().unwrap().to_string(),
                        }).collect();
                        sender.send(devices);
                    }
                }
            }
        }
    }
}

use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, State};
use tauri_specta::Event;
use tideperfect::{services::player::PlayerEvent, Event as RecvEvent, TidePerfect};
use tokio::sync::{broadcast, Mutex};
use tracing::{instrument, trace};

use crate::{dtos::{device::CommandDeviceDTO, track::TrackDTO}, error::ErrorDTO};

#[derive(Serialize, Deserialize, Debug, Clone, Type, Event)]
pub struct UpdatedCurrentTrack(Option<TrackDTO>);

#[derive(Serialize, Deserialize, Debug, Clone, Type, Event)]
pub struct UpdatedPauseState(bool);

#[derive(Serialize, Deserialize, Debug, Clone, Type, Event)]
pub struct UpdatedTrackProgress(u32);

// TODO: Error handling for event handlers - while the emit should work, I don't like the unwrap.
// TODO: Also, should we change how event handlers work? Perhaps each services handler should only
// recieve the corresponding event type, allowing us to remove the default case and give compiler
// errors if an event isn't handled.
pub fn handle_events(mut event_reciever: broadcast::Receiver<RecvEvent>, handle: &AppHandle) {
    let handle = handle.clone();
    tokio::spawn(async move {
        while let Ok(event) = event_reciever.recv().await {
            match event {
                RecvEvent::PlayerEvent(PlayerEvent::UpdatedCurrentTrack(track)) => {
                    UpdatedCurrentTrack(track.map(|t| TrackDTO::from(*t))).emit(&handle).unwrap();
                }
                RecvEvent::PlayerEvent(PlayerEvent::UpdatedPauseState(paused)) => {
                    UpdatedPauseState(paused).emit(&handle).unwrap();
                }
                RecvEvent::PlayerEvent(PlayerEvent::UpdatedTrackProgress(progress)) => {
                    UpdatedTrackProgress(progress).emit(&handle).unwrap();
                }
                _ => continue,
            }
        }
    });
}

#[tauri::command]
#[specta::specta]
#[instrument(skip(state))]
pub async fn play(state: State<'_, Mutex<TidePerfect>>) -> Result<(), ErrorDTO> {
    trace!("Got command: play");

    let state = state.lock().await;
    state.player_service.play().await?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
#[instrument(skip(state))]
pub async fn pause(state: State<'_, Mutex<TidePerfect>>) -> Result<(), ErrorDTO> {
    trace!("Got command: pause");

    let state = state.lock().await;
    state.player_service.pause().await?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
#[instrument(skip(state))]
pub async fn skip(state: State<'_, Mutex<TidePerfect>>) -> Result<(), ErrorDTO> {
    trace!("Got command: skip");

    let state = state.lock().await;
    state.player_service.skip().await?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
#[instrument(skip(state))]
pub async fn previous(state: State<'_, Mutex<TidePerfect>>) -> Result<(), ErrorDTO> {
    trace!("Got command: previous");

    let state = state.lock().await;
    state.player_service.previous().await?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
#[instrument(skip(state))]
pub async fn devices(state: State<'_, Mutex<TidePerfect>>) -> Result<Vec<CommandDeviceDTO>, ErrorDTO> {
    trace!("Got command: devices");

    let state = state.lock().await;
    let devices = state.player_service.devices().await?;
    let devices = devices.into_iter().map(|x| x.into()).collect();

    Ok(devices)
}

#[tauri::command]
#[specta::specta]
#[instrument(skip(state))]
pub async fn set_device(state: State<'_, Mutex<TidePerfect>>, device: String) -> Result<(), ErrorDTO> {
    trace!("Got command: set_device({device:?})");

    let state = state.lock().await;
    state.player_service.set_device(device).await?;

    Ok(())
}


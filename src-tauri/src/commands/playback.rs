use tauri::{async_runtime::Mutex, State};
use tracing::{error, instrument, trace};

use crate::{audio::{stream::stream_dash_audio, track::PlayerTrack}, error::AppError, state::AppState};

#[tauri::command]
#[specta::specta]
#[instrument(skip(state), err)]
pub async fn play_track(state: State<'_, Mutex<AppState>>, song_id: String) -> Result<(), AppError> {
    let mut state = state.lock().await;

    let song_id: u64 = song_id.parse().unwrap();

    let track = PlayerTrack::fetch(&state.client, song_id).await?;

    if let Err(error) = state.player.play_track(track) {
        error!("{error}");
    }

    Ok(())
}

#[tauri::command]
#[specta::specta]
#[instrument(skip(state), err)]
pub async fn stop_track(state: State<'_, Mutex<AppState>>) -> Result<(), AppError> {
    let mut state = state.lock().await;
    state.player.stop_track();
    Ok(())
}

#[tauri::command]
#[specta::specta]
#[instrument(skip(state), err)]
pub async fn devices(state: State<'_, Mutex<AppState>>) -> Result<Vec<String>, AppError> {
    let state = state.lock().await;

    let devices = state.player.devices().map_err(|e| AppError::Other(e.to_string()));
    trace!("Got devices: {devices:?}");
    devices
}

#[tauri::command]
#[specta::specta]
#[instrument(skip(state), err)]
pub async fn set_output_device(state: State<'_, Mutex<AppState>>, device: String) -> Result<(), AppError> {
    let mut state = state.lock().await;

    trace!("Setting device to {device}");
    state.player.set_device(&device).map_err(|e| AppError::Other(e.to_string()))
}

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

    // if let Err(error) = state.player.play_track(track) {
    //     error!("{error}");
    // }

    Ok(())
}


#[tauri::command]
#[specta::specta]
#[instrument(skip(state), err)]
pub async fn queue_track(state: State<'_, Mutex<AppState>>, song_id: String) -> Result<(), AppError> {
    let mut state = state.lock().await;

    let song_id: u64 = song_id.parse().unwrap();
    let track = PlayerTrack::fetch(&state.client, song_id).await?;

    let queue = state.player.queue.clone();
    let mut queue = queue.lock().await;
    queue.add(track);
    state.player.play_queue().await?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
#[instrument(skip(state), err)]
pub async fn play_queue(state: State<'_, Mutex<AppState>>) -> Result<(), AppError> {
    let state = state.lock().await;

    state.player.play_queue().await?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
#[instrument(skip(state), err)]
pub async fn skip_next(state: State<'_, Mutex<AppState>>) -> Result<(), AppError> {
    let state = state.lock().await;

    state.player.skip().await?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
#[instrument(skip(state), err)]
pub async fn previous(state: State<'_, Mutex<AppState>>) -> Result<(), AppError> {
    let state = state.lock().await;

    state.player.previous().await?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
#[instrument(skip(state), err)]
pub async fn pause(state: State<'_, Mutex<AppState>>) -> Result<(), AppError> {
    let state = state.lock().await;
    state.player.pause().await?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
#[instrument(skip(state), err)]
pub async fn resume(state: State<'_, Mutex<AppState>>) -> Result<(), AppError> {
    let state = state.lock().await;
    state.player.resume().await?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
#[instrument(skip(state), err)]
pub async fn stop_track(state: State<'_, Mutex<AppState>>) -> Result<(), AppError> {
    let mut state = state.lock().await;
    //state.player.stop_track();
    Ok(())
}

#[tauri::command]
#[specta::specta]
#[instrument(skip(state), err)]
pub async fn devices(state: State<'_, Mutex<AppState>>) -> Result<Vec<String>, AppError> {
    let state = state.lock().await;

    // let devices = state.player.devices().map_err(|e| AppError::Other(e.to_string()));
    // trace!("Got devices: {devices:?}");
    // devices
    Ok(Vec::new())
}

#[tauri::command]
#[specta::specta]
#[instrument(skip(state), err)]
pub async fn set_output_device(state: State<'_, Mutex<AppState>>, device: String) -> Result<(), AppError> {
    let mut state = state.lock().await;

    trace!("Setting device to {device}");
    // state.player.set_device(&device).map_err(|e| AppError::Other(e.to_string()))
    Ok(())
}

use tauri::{async_runtime::Mutex, State};
use tracing::instrument;

use crate::{error::AppError, models::track::Track, state::AppState};

#[tauri::command]
#[specta::specta]
#[instrument(skip(state), err)]
pub async fn fetch_track(state: State<'_, Mutex<AppState>>, song_id: String) -> Result<Track, AppError> {
    let state = state.lock().await;

    let song_id: u64 = song_id.parse().map_err(|_| AppError::StringToInt(song_id))?;
    let track = state.client.track(song_id).await?;

    Ok(track.into())
}

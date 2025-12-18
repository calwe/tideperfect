use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;
use tokio::sync::Mutex;
use tracing::{info, instrument, warn};
use strum_macros::AsRefStr;

use crate::{commands::{album, auth::AuthError}, error::AppError, models::track::Track, state::AppState};

#[derive(Debug, Type, Clone, Serialize, Deserialize)]
pub struct Album {
    id: String,
    title: String,
    quality: String,
}

#[tauri::command]
#[specta::specta]
#[instrument(skip(state), err)]
pub async fn favourite_albums(state: State<'_, Mutex<AppState>>) -> Result<Vec<Album>, AppError> {
    let state = state.lock().await;

    let albums = state.client.favorite_albums(None, None, None, None).await.unwrap();
    Ok(albums.items.iter()
        .map(|fav| {
            let album = fav.item.clone();
            Album {
                id: album.id.to_string(),
                title: album.title,
                quality: album.audio_quality.as_ref().to_string(),
            }
        })
        .collect())
}

#[tauri::command]
#[specta::specta]
#[instrument(skip(state), err)]
pub async fn album_tracks(state: State<'_, Mutex<AppState>>, id: String) -> Result<Vec<Track>, AppError> {
    let state = state.lock().await;

    let id: u64 = id.parse().unwrap();

    let tracks = state.client.album_tracks(id, None, None).await?;
    Ok(tracks.items.iter()
        .map(|track| track.clone().into())
        .collect())
}


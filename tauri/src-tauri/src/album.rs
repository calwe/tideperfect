use tauri::State;
use tideperfect::TidePerfect;
use tokio::sync::Mutex;
use tracing::instrument;

use crate::{dtos::{album::FavouriteAlbumDTO, track::TrackDTO}, error::ErrorDTO};

#[tauri::command]
#[specta::specta]
#[instrument(skip(state))]
pub async fn favourite_albums(state: State<'_, Mutex<TidePerfect>>) -> Result<Vec<FavouriteAlbumDTO>, ErrorDTO> {
    let state = state.lock().await;
    let favourite_albums = state.album_service.favourite_albums().await?;
    Ok(favourite_albums.into_iter()
        .map(|x| x.into())
        .collect()
    )
}

#[tauri::command]
#[specta::specta]
#[instrument(skip(state))]
pub async fn album_tracks(state: State<'_, Mutex<TidePerfect>>, id: String) -> Result<Vec<TrackDTO>, ErrorDTO> {
    let state = state.lock().await;
    let id = id.parse()?;
    let album_tracks = state.album_service.album_tracks(id).await?;
    Ok(album_tracks.into_iter()
        .map(|x| x.into())
        .collect()
    )
}


use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, State};
use tauri_specta::Event;
use tideperfect::{services::auth::AuthEvent, Event as RecvEvent, TidePerfect};
use tokio::sync::{broadcast, Mutex};
use tracing::instrument;

use crate::error::ErrorDTO;

#[derive(Serialize, Deserialize, Debug, Clone, Type, Event)]
pub struct LoggedIn;

pub fn handle_events(mut event_reciever: broadcast::Receiver<RecvEvent>, handle: &AppHandle) {
    let handle = handle.clone();
    tokio::spawn(async move {
        while let Ok(event) = event_reciever.recv().await {
            match event {
                RecvEvent::AuthEvent(AuthEvent::LoggedIn) => LoggedIn.emit(&handle).unwrap(),
                _ => continue,
            }
        }
    });
}

#[tauri::command]
#[specta::specta]
#[instrument(skip(state))]
pub async fn is_logged_in(state: State<'_, Mutex<TidePerfect>>) -> Result<bool, ErrorDTO> {
    let state = state.lock().await;
    Ok(state.auth_service.logged_in())
}

#[tauri::command]
#[specta::specta]
#[instrument(skip(state))]
pub async fn login(state: State<'_, Mutex<TidePerfect>>) -> Result<(), ErrorDTO> {
    let mut state = state.lock().await;
    Ok(state.auth_service.login().await?)
}

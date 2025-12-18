use specta::Type;
use tauri::{async_runtime::Mutex, State};
use thiserror::Error;
use tracing::{info, instrument, trace, warn};

use crate::{error::AppError, state::AppState, utils::persistence::{delete_auth_token, save_auth_token}};

#[derive(Debug, Error, Type)]
pub enum AuthError {
    #[error("Not logged in")]
    NotLoggedIn,
    #[error("No device code is set")]
    NoDeviceCode
}

#[tauri::command]
#[specta::specta]
#[instrument(skip(state), err)]
pub async fn get_username(state: State<'_, Mutex<AppState>>) -> Result<String, AppError> {
    let state = state.lock().await;

    if let Some(authz) = state.client.get_authz() {
        let response = state.client.user(authz.user_id).await?;
        info!("Logged in as {}", response.username);
        Ok(response.username)
    } else {
        warn!("User not logged in");
        Err(AuthError::NotLoggedIn)?
    }
}

#[tauri::command]
#[specta::specta]
#[instrument(skip(state), err)]
pub async fn start_authorization(state: State<'_, Mutex<AppState>>) -> Result<String, AppError> {
    let mut state = state.lock().await;
    let device_auth = state.client.device_authorization().await?;

    info!("Started login flow with code {}", device_auth.device_code);

    open::that(device_auth.url)?;

    state.device_code = Some(device_auth.device_code);
    Ok(device_auth.user_code)
}

#[tauri::command]
#[specta::specta]
#[instrument(skip(state), err)]
pub async fn authorize(state: State<'_, Mutex<AppState>>) -> Result<String, AppError> {
    let state = state.lock().await;

    if let Some(device_code) = &state.device_code {
        // Authorize with TIDAL - this also sets the auth on the client internally
        let authz_token = state.client.authorize(device_code, &state.client_secret).await?;

        // Save the full auth token to disk for persistence
        save_auth_token(&state.app_handle, &authz_token)?;

        let username = authz_token.user.username.clone();
        info!("User {username} authorised.");
        Ok(username)
    } else {
        Err(AuthError::NoDeviceCode)?
    }
}

#[tauri::command]
#[specta::specta]
#[instrument(skip(state), err)]
pub async fn logout(state: State<'_, Mutex<AppState>>) -> Result<(), AppError> {
    let state = state.lock().await;

    delete_auth_token(&state.app_handle)
}

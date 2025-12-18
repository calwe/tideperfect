use std::{fs, path::PathBuf};

use tauri::{AppHandle, Manager};
use tidalrs::AuthzToken;
use tracing::{info, instrument};

use crate::error::AppError;

// Auth persistence helper functions
#[instrument(err)]
fn get_auth_file_path(app_handle: &AppHandle) -> Result<PathBuf, AppError> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()?;

    fs::create_dir_all(&app_data_dir)?;

    Ok(app_data_dir.join("auth.json"))
}

#[instrument]
pub fn load_auth_token(app_handle: &AppHandle) -> Option<AuthzToken> {
    let path = get_auth_file_path(app_handle).ok()?;
    let contents = fs::read_to_string(path).ok()?;
    serde_json::from_str(&contents).ok()
}

#[instrument(err)]
pub fn save_auth_token(app_handle: &AppHandle, authz_token: &AuthzToken) -> Result<(), AppError> {
    let path = get_auth_file_path(app_handle)?;
    let json = serde_json::to_string_pretty(authz_token)?;
    fs::write(path, json)?;
    Ok(())
}

#[instrument(err)]
pub fn delete_auth_token(app_handle: &AppHandle) -> Result<(), AppError> {
    let path = get_auth_file_path(app_handle)?;
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}


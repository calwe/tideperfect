use std::{io, string::FromUtf8Error};

use base64::DecodeError;
use dash_mpd::DashMpdError;
use serde::{Serialize, Serializer};
use specta::Type;
use thiserror::Error;

use crate::{audio::track::PlayerTrackError, commands::auth::AuthError};

#[derive(Debug, Error, Type)]
pub enum AppError {
    #[error(transparent)]
    Auth(#[from] AuthError),
    #[error(transparent)]
    PlayerTrack(#[from] PlayerTrackError),

    #[error("Tauri error: {0}")]
    Tauri(String),
    #[error("IO error: {0}")]
    Io(String),
    #[error("Serialization error: {0}")]
    Serde(String),
    #[error("Tidal error: {0}")]
    Tidal(String),
    #[error("UTF-8 error: {0}")]
    Utf8(String),
    #[error("DASH MPD error: {0}")]
    DashMpd(String),
    #[error("Base64 decode error: {0}")]
    Base64Decode(String),
    #[error("Failed to parse string into integer: {0}")]
    StringToInt(String),

    #[error("Other error: {0}")]
    Other(String),
}

// Direct From implementations - convert external errors to strings
impl From<tauri::Error> for AppError {
    fn from(err: tauri::Error) -> Self {
        AppError::Tauri(err.to_string())
    }
}

impl From<io::Error> for AppError {
    fn from(err: io::Error) -> Self {
        AppError::Io(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Serde(err.to_string())
    }
}

impl From<tidalrs::Error> for AppError {
    fn from(err: tidalrs::Error) -> Self {
        AppError::Tidal(err.to_string())
    }
}

impl From<FromUtf8Error> for AppError {
    fn from(err: FromUtf8Error) -> Self {
        AppError::Utf8(err.to_string())
    }
}

impl From<DashMpdError> for AppError {
    fn from(err: DashMpdError) -> Self {
        AppError::DashMpd(err.to_string())
    }
}

impl From<DecodeError> for AppError {
    fn from(err: DecodeError) -> Self {
        AppError::Base64Decode(err.to_string())
    }
}

impl From<String> for AppError {
    fn from(value: String) -> Self {
        Self::Other(value)
    }
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

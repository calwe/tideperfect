use std::num::ParseIntError;

use serde::{Deserialize, Serialize};
use snafu::Report;
use specta::Type;
use tideperfect::{services::{album::AlbumServiceError, auth::AuthServiceError, player::PlayerServiceError, queue::QueueServiceError}, TidePerfectError};
use tracing::error;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ErrorDTO {
    error: String,
}

impl From<TidePerfectError> for ErrorDTO {
    fn from(value: TidePerfectError) -> Self {
        let report = Report::from_error(&value);
        error!("TidePerfect returned an error:\n{}", report);

        Self {
            error: value.to_string(),
        }
    }
}

impl From<AuthServiceError> for ErrorDTO {
    fn from(value: AuthServiceError) -> Self {
        let report = Report::from_error(&value);
        error!("TidePerfect returned an error:\n{}", report);

        Self {
            error: value.to_string(),
        }
    }
}

impl From<AlbumServiceError> for ErrorDTO {
    fn from(value: AlbumServiceError) -> Self {
        let report = Report::from_error(&value);
        error!("TidePerfect returned an error:\n{}", report);

        Self {
            error: value.to_string(),
        }
    }
}

impl From<QueueServiceError> for ErrorDTO {
    fn from(value: QueueServiceError) -> Self {
        let report = Report::from_error(&value);
        error!("TidePerfect returned an error:\n{}", report);

        Self {
            error: value.to_string(),
        }
    }
}

impl From<PlayerServiceError> for ErrorDTO {
    fn from(value: PlayerServiceError) -> Self {
        let report = Report::from_error(&value);
        error!("TidePerfect returned an error:\n{}", report);

        Self {
            error: value.to_string(),
        }
    }
}

impl From<ParseIntError> for ErrorDTO {
    fn from(value: ParseIntError) -> Self {
        let report = Report::from_error(&value);
        error!("TidePerfect returned an error:\n{}", report);

        Self {
            error: value.to_string(),
        }
    }
}

use std::sync::Arc;

use snafu::{ResultExt, Snafu};
use tidalrs::{TidalApiError, TidalClient};

pub use tidalrs::{Track, MediaMetadata, AudioQuality};
use tracing::instrument;

pub struct TrackService {
    tidal_client: Arc<TidalClient>,
}

impl TrackService {
    #[instrument(skip(tidal_client))]
    pub fn new(tidal_client: Arc<TidalClient>) -> Self {
        Self {
            tidal_client
        }
    }

    /// Get the lyrics for a track
    #[instrument(skip(self), err)]
    pub async fn lyrics(&self, id: u64) -> Result<Option<String>, TrackServiceError> {
        // TODO: Use subtitles instead of lyrics to have timed lyrics, also dont ignore direction
        let resp = self.tidal_client.lyrics(id).await;
        let resp = match resp {
            Err(tidalrs::Error::TidalApiError(
                TidalApiError {
                    status: 404,
                    ..
                }
            )) => return Ok(None),
            _ => resp
        };
        let lyrics = resp.context(LyricsSnafu { id })?.lyrics;
        Ok(Some(lyrics))
    }
}

#[derive(Debug, Snafu)]
pub enum  TrackServiceError {
    #[snafu(display("Could not get lyrics for track #{id}"))]
    Lyrics {
        source: tidalrs::Error,
        id: u64,
    },
}

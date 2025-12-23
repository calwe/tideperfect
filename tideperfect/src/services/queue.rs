use std::sync::{Arc};

use snafu::{ResultExt, Snafu};
use tidalrs::TidalClient;
use tokio::sync::{broadcast, Mutex};
use tracing::{instrument, trace};

use crate::{audio::{queue::{Queue, QueueError}, track::{Track, TrackError}}, Event};

pub use crate::audio::queue::{QueueEvent, QueueEventDiscriminants};

pub struct QueueService {
    tidal_client: Arc<TidalClient>,
    queue: Arc<Mutex<Queue>>,
}

impl QueueService {
    #[instrument(skip(tidal_client))]
    pub fn init(tidal_client: Arc<TidalClient>, event_emitter: broadcast::Sender<Event>) -> (Self, Arc<Mutex<Queue>>) {
        trace!("Initialising QueueService");
        let queue = Arc::new(Mutex::new(Queue::new(event_emitter)));
        (
            Self { 
                tidal_client,
                queue: queue.clone(),
            },
            queue.clone(),
        )
    }

    #[instrument(skip(self))]
    pub async fn queue_track(&self, id: u64) -> Result<(), QueueServiceError> {
        trace!("Queueing track #{id}");
        let track = Track::fetch(&self.tidal_client, id).await.context(FetchTrackSnafu { id })?;
        self.queue.lock().await.add(track).context(AddTrackSnafu { id })?;

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn queue_album(&self, id: u64) -> Result<(), QueueServiceError> {
        trace!("Queueing album #{id}");
        let album = self.tidal_client.album_tracks(id, None, None).await.context(FetchAlbumTracksSnafu { id })?;
        for track in album.items {
            let track = Track::fetch_from_track(&self.tidal_client, &track).await
                .context(FetchTrackSnafu { id: track.id })?;
            self.queue.lock().await.add(track).context(AddTrackSnafu { id })?;
        }

        Ok(())
    }
}

#[derive(Debug, Snafu)]
pub enum QueueServiceError {
    #[snafu(display("Failed to fetch track: {id}"))]
    FetchTrack {
        id: u64,
        source: TrackError,
    },
    #[snafu(display("Failed to fetch album tracks: {id}"))]
    FetchAlbumTracks {
        id: u64,
        source: tidalrs::Error,
    },
    #[snafu(display("Failed to add #{id} to queue"))]
    AddTrack {
        id: u64,
        source: QueueError,
    },
}

use std::collections::VecDeque;

use snafu::{ResultExt, Snafu};
use tokio::sync::broadcast;
use tracing::{instrument, trace};
use strum_macros::EnumDiscriminants;

use crate::{audio::track::Track, Event};

#[derive(Debug)]
pub struct Queue {
    event_emitter: broadcast::Sender<Event>,
    tracks: VecDeque<Track>,
}

impl Queue {
    #[instrument]
    pub fn new(event_emitter: broadcast::Sender<Event>) -> Self {
        trace!("Creating Queue");
        Self {
            event_emitter,
            tracks: VecDeque::new(),
        }
    }

    #[instrument]
    pub fn add(&mut self, track: Track) -> Result<(), QueueError> {
        trace!("Adding {track:?} to queue");

        self.tracks.push_back(track);

        let event = Event::QueueEvent(QueueEvent::QueueUpdated(self.into()));
        self.event_emitter.send(event.clone()).context(SendEventSnafu { event })?;

        Ok(())
    }

    pub fn deque(&mut self) -> Result<Option<Track>, QueueError> {
        let result = self.tracks.pop_front();

        trace!("Dequeued {result:?} from queue");

        let event = Event::QueueEvent(QueueEvent::QueueUpdated(self.into()));
        self.event_emitter.send(event.clone()).context(SendEventSnafu { event })?;

        Ok(result)
    }
}

impl From<&mut Queue> for Vec<tidalrs::Track> {
    fn from(val: &mut Queue) -> Self {
        val.tracks.iter().map(|track| track.track.clone()).collect()
    }
}

#[derive(Debug, Clone, EnumDiscriminants)]
pub enum QueueEvent {
    QueueUpdated(Vec<tidalrs::Track>),
}

#[derive(Debug, Snafu)]
pub enum QueueError {
    #[snafu(display("Failed to send event: {event:?}"))]
    SendEvent {
        source: broadcast::error::SendError<Event>,
        event: Event,
    },
}

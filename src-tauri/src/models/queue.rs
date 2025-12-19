use std::collections::VecDeque;

use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::AppHandle;
use tauri_specta::Event;

use crate::{audio::track::PlayerTrack, models::track::Track};

#[derive(Serialize, Deserialize, Debug, Clone, Type, Event)]
pub struct UpdatedQueueEvent(Vec<Track>);

pub struct Queue {
    tracks: VecDeque<PlayerTrack>,
    app_handle: AppHandle,
}

impl Queue {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            tracks: VecDeque::new(),
            app_handle,
        }
    }

    pub fn add(&mut self, track: PlayerTrack) {
        self.tracks.push_back(track);
        UpdatedQueueEvent(self.into()).emit(&self.app_handle).unwrap();
    }

    pub fn deque(&mut self) -> Option<PlayerTrack> {
        let result = self.tracks.pop_front();
        UpdatedQueueEvent(self.into()).emit(&self.app_handle).unwrap();
        result
    }
}

impl From<&mut Queue> for Vec<Track> {
    fn from(val: &mut Queue) -> Self {
        val.tracks.iter().map(|track| track.track.clone()).collect()
    }
}

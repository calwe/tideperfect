use std::collections::VecDeque;

use crate::audio::track::PlayerTrack;

pub struct Queue {
    tracks: VecDeque<PlayerTrack>,
}

impl Queue {
    pub fn new() -> Self {
        Self {
            tracks: VecDeque::new(),
        }
    }

    pub fn add(&mut self, track: PlayerTrack) {
        self.tracks.push_back(track);
    }

    pub fn deque(&mut self) -> Option<PlayerTrack> {
        self.tracks.pop_front()
    }
}

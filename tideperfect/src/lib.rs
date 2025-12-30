use std::{env, path::Path, sync::Arc};

use snafu::{ResultExt, Snafu};
use strum_macros::EnumDiscriminants;
use tokio::sync::broadcast;

use crate::{audio::{player::PlayerEvent, queue::QueueEvent}, services::{album::AlbumService, auth::{AuthEvent, AuthService, AuthServiceError},
        player::{PlayerService, PlayerServiceError}, queue::QueueService, track::TrackService}, utils::persistence::{PersistanceError, Persistence}};

use dotenvy::dotenv;

mod audio;
pub mod services;
pub mod utils;

#[derive(Debug, Clone, EnumDiscriminants)]
pub enum Event {
    AuthEvent(AuthEvent),
    QueueEvent(QueueEvent),
    PlayerEvent(PlayerEvent),
}

pub struct TidePerfect {
    pub auth_service: AuthService,
    pub album_service: AlbumService,
    pub queue_service: QueueService,
    pub player_service: PlayerService,
    pub track_service: TrackService,
}

impl TidePerfect {
    pub fn init(data_dir: &Path, event_emitter: broadcast::Sender<Event>) -> Result<Self, TidePerfectError> {
        dotenv().ok();

        let client_id = env::var("TIDAL_CLIENT_ID").unwrap_or(dotenvy_macro::dotenv!("TIDAL_CLIENT_ID").to_owned());
        let client_secret = env::var("TIDAL_CLIENT_SECRET").unwrap_or(dotenvy_macro::dotenv!("TIDAL_CLIENT_SECRET").to_owned());

        let persistence = Arc::new(Persistence::new(data_dir).context(PersistenceSnafu)?);
        
        let (auth_service, tidal_client) = AuthService::init(persistence.clone(), event_emitter.clone(), &client_id, &client_secret);
        let (queue_service, queue) = QueueService::init(tidal_client.clone(), event_emitter.clone());

        let album_service = AlbumService::new(tidal_client.clone());
        let player_service = PlayerService::init_default_output(queue.clone(), event_emitter.clone()).context(PlayerServiceSnafu)?;
        let track_service = TrackService::new(tidal_client.clone());

        Ok(Self {
            auth_service,
            album_service,
            queue_service,
            player_service,
            track_service,
        })
    }
}

#[derive(Debug, Snafu)]
pub enum TidePerfectError {
    PersistenceError {
        source: PersistanceError,
    },
    AuthServiceError {
        source: AuthServiceError,
    },
    PlayerServiceError {
        source: PlayerServiceError,
    },
}

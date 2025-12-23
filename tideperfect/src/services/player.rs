use std::sync::Arc;

use cpal::{default_host, traits::{DeviceTrait, HostTrait}};
use snafu::Snafu;
use tokio::sync::{broadcast, mpsc, Mutex};
use tracing::{instrument, trace};

use crate::{audio::{player::{player_loop, PlayerCommand}, queue::Queue, track::Track}, Event};

pub use crate::audio::player::{PlayerEvent, PlayerEventDiscriminants};

pub struct PlayerService {
    control_tx: mpsc::Sender<PlayerCommand>,
    pub queue: Arc<Mutex<Queue>>,
    pub played: Arc<Mutex<Vec<Track>>>,
}

impl PlayerService {
    #[instrument(skip_all, err)]
    pub fn init_default_output(queue: Arc<Mutex<Queue>>, event_emitter: broadcast::Sender<Event>) -> Result<Self, PlayerServiceError> {
        let host = default_host();
        let device = host.default_output_device().ok_or(PlayerServiceError::NoDefaultDevice)?;
        trace!("Using device with name: {:?}", device.name());

        let played = Arc::new(Mutex::new(Vec::new()));
        let (control_tx, control_rx) = mpsc::channel(32);

        tokio::spawn(player_loop(
                control_rx, 
                control_tx.clone(),
                event_emitter,
                host, 
                device.clone(),
                queue.clone(),
                played.clone()
        ));

        Ok(Self {
            control_tx,
            queue,
            played,
        })
    }

    pub async fn play(&self) -> Result<(), PlayerServiceError> {
        self.control_tx.send(PlayerCommand::Play).await
            .map_err(|_| PlayerServiceError::BackgroundThreadDied)?;
        Ok(())
    }

    pub async fn pause(&self) ->  Result<(), PlayerServiceError> {
        self.control_tx.send(PlayerCommand::Pause).await
            .map_err(|_| PlayerServiceError::BackgroundThreadDied)?;
        Ok(())
    }

    pub async fn skip(&self) -> Result<(), PlayerServiceError> {
        self.control_tx.send(PlayerCommand::Skip).await
            .map_err(|_| PlayerServiceError::BackgroundThreadDied)?;
        Ok(())
    }

    pub async fn previous(&self) -> Result<(), PlayerServiceError> {
        self.control_tx.send(PlayerCommand::Previous).await
            .map_err(|_| PlayerServiceError::BackgroundThreadDied)?;
        Ok(())
    }
}

#[derive(Debug, Snafu)]
pub enum PlayerServiceError {
    #[snafu(display("No default device"))]
    NoDefaultDevice,
    #[snafu(display("Background thread died"))]
    BackgroundThreadDied,
}

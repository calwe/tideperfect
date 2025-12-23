use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, State};
use tauri_specta::Event;
use tideperfect::{services::queue::QueueEvent, Event as RecvEvent, TidePerfect};
use tokio::sync::{broadcast, Mutex};
use tracing::{instrument, trace};

use crate::{dtos::track::TrackDTO, error::ErrorDTO};

#[derive(Serialize, Deserialize, Debug, Clone, Type, Event)]
pub struct QueueUpdated(Vec<TrackDTO>);

pub fn handle_events(mut event_reciever: broadcast::Receiver<RecvEvent>, handle: &AppHandle) {
    let handle = handle.clone();
    tokio::spawn(async move {
        while let Ok(event) = event_reciever.recv().await {
            match event {
                RecvEvent::QueueEvent(QueueEvent::QueueUpdated(queue)) => {
                    let queue = queue.into_iter().map(|t| t.into()).collect();
                    QueueUpdated(queue).emit(&handle).unwrap();
                }
                _ => continue,
            }
        }
    });
}

#[tauri::command]
#[specta::specta]
#[instrument(skip(state))]
pub async fn queue_track(state: State<'_, Mutex<TidePerfect>>, id: String) -> Result<(), ErrorDTO> {
    trace!("Got command: queue_track({id})");

    let state = state.lock().await;
    let id = id.parse()?;
    state.queue_service.queue_track(id).await?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
#[instrument(skip(state))]
pub async fn queue_album(state: State<'_, Mutex<TidePerfect>>, id: String) -> Result<(), ErrorDTO> {
    trace!("Got command: queue_album({id})");

    let state = state.lock().await;
    let id = id.parse()?;
    state.queue_service.queue_album(id).await?;

    Ok(())
}


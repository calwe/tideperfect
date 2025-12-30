use tauri::State;
use tideperfect::TidePerfect;
use tokio::sync::Mutex;
use tracing::{error, info, instrument};

use crate::error::ErrorDTO;

#[tauri::command]
#[specta::specta]
#[instrument(skip(state))]
pub async fn lyrics(state: State<'_, Mutex<TidePerfect>>, id: &str) -> Result<Option<String>, ErrorDTO> {
    error!("LYRICS");
    let state = state.lock().await;
    let id = id.parse()?;
    let resp = Ok(state.track_service.lyrics(id).await?);
    info!("resp: {resp:?}");
    resp
}

use tauri::AppHandle;
use tidalrs::TidalClient;

use crate::audio::player::Player;

pub struct AppState {
    pub client: TidalClient,
    pub client_secret: String,
    pub device_code: Option<String>,
    pub app_handle: AppHandle,
    pub player: Player,
}

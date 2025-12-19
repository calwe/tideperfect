use std::env;
use dotenv::dotenv;
#[cfg(debug_assertions)]
use specta_typescript::BigIntExportBehavior;
use tauri::{async_runtime::Mutex, Manager};
use tauri_specta::{collect_commands, collect_events, Builder};
use specta_typescript::Typescript;
use tidalrs::{DeviceType, TidalClient};
use tracing::trace;
use tracing_subscriber::EnvFilter;

use crate::{audio::player::Player, commands::*, models::queue, state::AppState, utils::persistence::load_auth_token};

mod audio;
mod commands;
mod error;
mod models;
mod state;
mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    dotenv().ok();

    #[cfg(feature = "devtools")]
    let devtools = tauri_plugin_devtools::init();

    #[cfg(not(feature = "devtools"))]
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let client_id = env::var("TIDAL_CLIENT_ID").expect("TIDAL_CLIENT_ID must be set");
    let client_secret = env::var("TIDAL_CLIENT_SECRET").expect("TIDAL_CLIENT_SECRET must be set");

    let specta_builder = Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            album::favourite_albums, album::album_tracks,
            auth::start_authorization, auth::authorize, auth::get_username, auth::logout,
            playback::play_track, playback::stop_track, playback::devices, playback::set_output_device,
            playback::queue_track, playback::play_queue, playback::skip_next, playback::pause, playback::resume,
            track::fetch_track,
        ])
        .events(collect_events![
            queue::UpdatedQueueEvent,
            audio::track::CurrentTrackEvent,
            audio::player::PlaybackStateEvent,
        ]);

    #[cfg(debug_assertions)] // <- Only export on non-release builds
    specta_builder
        .export(Typescript::default().bigint(BigIntExportBehavior::String), "../src/bindings.ts")
        .expect("Failed to export typescript bindings");

    let tauri_builder = tauri::Builder::default();

    #[cfg(feature = "devtools")]
    let tauri_builder = tauri_builder.plugin(devtools);

    tauri_builder
        .invoke_handler(specta_builder.invoke_handler())
        .setup(move |app| {
            let app_handle = app.handle().clone();

            // Try to load saved auth token
            let saved_auth = load_auth_token(&app_handle);

            // Create TidalClient, using saved auth if available
            let client = if let Some(auth_token) = &saved_auth {
                // Extract Authz from the saved AuthzToken
                if let Some(authz) = auth_token.authz() {
                    TidalClient::new(client_id.clone()).with_authz(authz).with_device_type(DeviceType::Browser)
                } else {
                    TidalClient::new(client_id.clone()).with_device_type(DeviceType::Browser)
                }
            } else {
                TidalClient::new(client_id.clone()).with_device_type(DeviceType::Browser)
            };

            let player = Player::init_default_output(app.handle().clone())?;

            app.manage(Mutex::new(AppState {
                client,
                client_secret: client_secret.clone(),
                device_code: None,
                app_handle,
                player,
            }));

            specta_builder.mount_events(app);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

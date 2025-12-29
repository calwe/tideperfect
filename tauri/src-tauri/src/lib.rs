use std::env;
#[cfg(debug_assertions)]
use specta_typescript::BigIntExportBehavior;
use tauri::{async_runtime::Mutex, Manager};
use tauri_specta::{collect_commands, collect_events, Builder};
use specta_typescript::Typescript;
use tideperfect::{services::{auth::AuthEventDiscriminants, player::PlayerEventDiscriminants, queue::QueueEventDiscriminants}, Event, EventDiscriminants, TidePerfect, TidePerfectError};
use tokio::sync::broadcast;
use tracing::trace;
use tracing_subscriber::EnvFilter;

mod album;
mod auth;
mod dtos;
mod error;
mod player;
mod queue;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<(), TidePerfectError> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let specta_builder = Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            auth::is_logged_in, auth::login,
            album::favourite_albums, album::album_tracks,
            queue::queue_track, queue::queue_album,
            player::play, player::pause, player::skip, player::previous,
            player::devices, player::set_device,
        ])
        .events(collect_events![
            auth::LoggedIn,
            queue::QueueUpdated,
            player::UpdatedCurrentTrack, player::UpdatedPauseState, player::UpdatedTrackProgress
        ]);

    #[cfg(debug_assertions)]
    specta_builder
        .export(Typescript::default().bigint(BigIntExportBehavior::String), "../src/bindings.ts")
        .expect("Failed to export typescript bindings");

    let tauri_builder = tauri::Builder::default();

    tauri_builder
        .invoke_handler(specta_builder.invoke_handler())
        .setup(move |app| {
            let app_handle = app.handle().clone();

            let (event_emitter, event_reciever) = broadcast::channel(32);
            let tideperfect = TidePerfect::init(&app_handle.path().data_dir().unwrap(), event_emitter.clone())?;

            let log_event_filter = vec![
                EventFilter::PlayerEvent(PlayerEventDiscriminants::UpdatedTrackProgress),
                EventFilter::QueueEvent(QueueEventDiscriminants::QueueUpdated),
            ];

            log_events(event_reciever, log_event_filter);
            auth::handle_events(event_emitter.clone().subscribe(), &app_handle);
            queue::handle_events(event_emitter.clone().subscribe(), &app_handle);
            player::handle_events(event_emitter.clone().subscribe(), &app_handle);

            app.manage(Mutex::new(tideperfect));

            specta_builder.mount_events(app);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}

#[derive(PartialEq)]
pub enum EventFilter {
    Category(EventDiscriminants),
    AuthEvent(AuthEventDiscriminants),
    PlayerEvent(PlayerEventDiscriminants),
    QueueEvent(QueueEventDiscriminants),
}

pub fn log_events(mut event_reciever: broadcast::Receiver<Event>, filter: Vec<EventFilter>) {
    tokio::spawn(async move {
        while let Ok(event) = event_reciever.recv().await {
            if filter.contains(&EventFilter::Category(EventDiscriminants::from(&event))) {
                continue;
            }

            let filtered = match &event {
                Event::AuthEvent(inner) => 
                    filter.contains(&EventFilter::AuthEvent(AuthEventDiscriminants::from(inner))),
                Event::PlayerEvent(inner) => 
                    filter.contains(&EventFilter::PlayerEvent(PlayerEventDiscriminants::from(inner))),
                Event::QueueEvent(inner) => 
                    filter.contains(&EventFilter::QueueEvent(QueueEventDiscriminants::from(inner))),
            };

            if !filtered {
                trace!("Recieved event: {event:?}");
            }
        }
    });
}

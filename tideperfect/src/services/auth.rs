use std::{io, sync::{atomic::{AtomicBool, Ordering}, Arc}, time::Duration};
use snafu::{ResultExt, Snafu};
use tidalrs::{AuthzToken, DeviceType, TidalApiError, TidalClient};
use tokio::{sync::broadcast, time::sleep};
use tracing::{instrument, trace};
use strum_macros::EnumDiscriminants;

use crate::{utils::persistence::{PersistanceError, Persistence, PersistenceContext}, Event};

pub struct AuthService {
    tidal_client: Arc<TidalClient>,
    persistence: Arc<Persistence>,
    event_emitter: broadcast::Sender<Event>,
    logged_in: Arc<AtomicBool>,
    client_secret: String,
}

#[derive(Debug, Clone, EnumDiscriminants)]
pub enum AuthEvent {
    LoggedIn,
}

impl std::fmt::Debug for AuthService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthService").field("logged_in", &self.logged_in).finish()
    }
}

impl PersistenceContext for AuthzToken {}

impl AuthService {
    #[instrument(skip(event_emitter))]
    pub fn init(
        persistence: Arc<Persistence>, 
        event_emitter: broadcast::Sender<Event>,
        client_id: &str, 
        client_secret: &str
    ) -> (Self, Arc<TidalClient>) {
        let (client, logged_in) = if let Ok(auth_token) = &persistence.load::<AuthzToken>() {
            if let Some(authz) = auth_token.authz() {
                (
                    TidalClient::new(client_id.to_owned()).with_authz(authz).with_device_type(DeviceType::Browser),
                    true,
                )
            } else {
                (
                    TidalClient::new(client_id.to_owned()).with_device_type(DeviceType::Browser),
                    false,
                )
            }
        } else {
            (
                TidalClient::new(client_id.to_owned()).with_device_type(DeviceType::Browser),
                false,
            )
        };

        let client = Arc::new(client);
        let logged_in = Arc::new(AtomicBool::new(logged_in));

        (
            Self { 
                tidal_client: client.clone(), 
                client_secret: client_secret.to_owned(),
                persistence, 
                event_emitter,
                logged_in, 
            },
            client.clone(),
        )
    }

    #[instrument]
    pub fn logged_in(&self) -> bool {
        trace!("Logged in");
        self.logged_in.load(Ordering::SeqCst)
    }

    #[instrument(err)]
    pub async fn login(&mut self) -> Result<String, AuthServiceError> {
        let device_auth = self.tidal_client.device_authorization().await
            .context(DeviceAuthorizationSnafu)?;

        trace!("Started login flow at {}", device_auth.url);

        let tidal_client = self.tidal_client.clone();
        let device_code = device_auth.device_code;
        let client_secret = self.client_secret.clone();
        let persistence = self.persistence.clone();
        let event_emitter = self.event_emitter.clone();
        let logged_in = self.logged_in.clone();

        tokio::spawn(async move {
            let poll_interval = Duration::from_secs(1);
            
            loop {
                sleep(poll_interval).await;

                let authorized = tidal_client.authorize(&device_code, &client_secret).await;
                match authorized {
                    Ok(token) => {
                        trace!("User logged in");
                        event_emitter.send(Event::AuthEvent(AuthEvent::LoggedIn))
                            .context(SendEventSnafu { event: Event::AuthEvent(AuthEvent::LoggedIn) })?;
                        persistence.store(&token).context(StoreTokenSnafu)?;
                        logged_in.store(true, Ordering::SeqCst);
                        return Ok(())
                    }
                    Err(tidalrs::Error::TidalApiError(TidalApiError {
                        status: 400,
                        sub_status: 1002,
                        user_message: _,
                    })) => continue,
                    Err(e) => {
                        return Err(AuthServiceError::Polling { source: e })
                    }
                }
            }
        });

        Ok(device_auth.user_code)
    }
}

#[derive(Debug, Snafu)]
pub enum AuthServiceError {
    #[snafu(display("Could not store auth token"))]
    StoreToken {
        source: PersistanceError,
    },
    #[snafu(display("Could not start device authorization flow"))]
    DeviceAuthorization {
        source: tidalrs::Error,
    },
    #[snafu(display("Could not open browser"))]
    OpenBrowser {
        source: io::Error,
    },
    #[snafu(display("Error while polling tidal API"))]
    Polling {
        source: tidalrs::Error,
    },
    #[snafu(display("Failed to send event '{event:?}'"))]
    SendEvent {
        event: Event,
        source: broadcast::error::SendError<Event>,
    },
}

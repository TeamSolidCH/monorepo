pub mod update_calendar_event;
pub mod verify_calendar_id;

use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use google_calendar3::api::Event;
use google_calendar3::hyper::client::HttpConnector;
use google_calendar3::{hyper, hyper_rustls, oauth2, CalendarHub, Result};
use std::collections::BTreeMap;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::events::UpdateCalendarEvent;
use crate::events::VerifyCalendarEvent;

pub struct GCalendar {
    pub hub: CalendarHub<hyper_rustls::HttpsConnector<HttpConnector>>,
    pub db: Pool<ConnectionManager<PgConnection>>,
    events_cache: BTreeMap<String, Vec<Event>>,
    senders: GCalendarChannelSenders,
}

pub struct GCalendarChannelReceivers {
    pub verify_calendar_rx: Receiver<VerifyCalendarEvent>,
}

#[derive(Clone)]
pub struct GCalendarChannelSenders {
    pub update_calendar_tx: Sender<UpdateCalendarEvent>,
}

impl Clone for GCalendar {
    fn clone(&self) -> Self {
        Self {
            hub: self.hub.clone(),
            db: self.db.clone(),
            events_cache: self.events_cache.clone(),
            senders: self.senders.clone(),
        }
    }
}

impl GCalendar {
    pub async fn new(
        db: Pool<ConnectionManager<PgConnection>>,
        senders: GCalendarChannelSenders,
    ) -> Result<GCalendar> {
        let env = std::env::var("GOOGLE_CALENDAR_SERVICE_FILE")
            .expect("GOOGLE_CALENDAR_SERVICE_FILE not set");

        let service = oauth2::read_service_account_key(env)
            .await
            .expect("Unable to load service account file");

        let authenticator = oauth2::ServiceAccountAuthenticator::builder(service)
            .build()
            .await
            .expect("authenticator failed");

        let hub = CalendarHub::new(
            hyper::Client::builder().build(
                hyper_rustls::HttpsConnectorBuilder::new()
                    .with_native_roots()?
                    .https_only()
                    .enable_http1()
                    .build(),
            ),
            authenticator,
        );
        Ok(GCalendar {
            db,
            hub,
            events_cache: BTreeMap::new(),
            senders,
        })
    }

    pub fn init_threads(self, receivers: GCalendarChannelReceivers) -> Self {
        self.new_update_calendars_thread()
            .new_verify_calendar_id_thread(receivers.verify_calendar_rx)
    }
}

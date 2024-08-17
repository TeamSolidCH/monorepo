extern crate google_calendar3 as calendar3;

use crate::models::*;

use calendar3::hyper::client::HttpConnector;
use calendar3::{chrono, hyper, hyper_rustls, oauth2, CalendarHub, Result};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use tokio::sync::mpsc::Sender;

pub struct GCalendar {
    pub hub: CalendarHub<hyper_rustls::HttpsConnector<HttpConnector>>,
    pub db: Pool<ConnectionManager<PgConnection>>,
}

pub struct UpdateCalendarEvent {
    pub calendar_id: String,
    pub new_events: Vec<calendar3::api::Event>,
}

impl GCalendar {
    pub async fn new(db: Pool<ConnectionManager<PgConnection>>) -> Result<GCalendar> {
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
                    .https_or_http()
                    .enable_http1()
                    .build(),
            ),
            authenticator,
        );
        Ok(GCalendar { db, hub })
    }

    pub async fn update_calendars(&self, sender: Sender<UpdateCalendarEvent>) {
        use crate::schema::calendars::dsl::*;
        let db = &mut self.db.clone().get().unwrap();
        let db_calendars = calendars
            .select(Calendar::as_select())
            .load(db)
            .expect("Unable to get calendars");
        for calendar in db_calendars {
            let calendar_id = calendar.googleid.clone();
            let sender = sender.clone();
            let events = self
                .hub
                .events()
                .list(&calendar_id)
                .time_min(chrono::Utc::now())
                .doit()
                .await
                .expect("Unable to get events")
                .1;
            sender
                .send(UpdateCalendarEvent {
                    calendar_id,
                    new_events: events.items.unwrap_or_default(),
                })
                .await
                .expect("Unable to send events");
        }
    }

    pub fn compare_event(a: &calendar3::api::Event, b: &calendar3::api::Event) -> bool {
        a.id == b.id
            && a.summary == b.summary
            && a.description == b.description
            && a.location == b.location
            && a.start.as_ref().unwrap().date_time.as_ref().unwrap()
                == b.start.as_ref().unwrap().date_time.as_ref().unwrap()
            && a.end.as_ref().unwrap().date_time.as_ref().unwrap()
                == b.end.as_ref().unwrap().date_time.as_ref().unwrap()
    }
}

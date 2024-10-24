/*
Calendarbot  Copyright (C) 2023 Zbinden Yohan

This program comes with ABSOLUTELY NO WARRANTY; for details type `show w'.
This is free software, and you are welcome to redistribute it
 */

pub mod update_calendar_event;
pub mod worker_thread;

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::AsyncPgConnection;
use google_calendar3::api::Event;
use google_calendar3::hyper::client::HttpConnector;
use google_calendar3::{hyper, hyper_rustls, oauth2, CalendarHub, Result};
use std::collections::BTreeMap;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::events::{CalendarCommands, UpdateCalendarEvent};

pub struct GCalendar {
    pub hub: CalendarHub<hyper_rustls::HttpsConnector<HttpConnector>>,
    pub db: Pool<AsyncPgConnection>,
    events_cache: BTreeMap<String, Vec<Event>>,
    calendar_update_tx: Sender<UpdateCalendarEvent>,
}

impl Clone for GCalendar {
    fn clone(&self) -> Self {
        Self {
            hub: self.hub.clone(),
            db: self.db.clone(),
            events_cache: self.events_cache.clone(),
            calendar_update_tx: self.calendar_update_tx.clone(),
        }
    }
}

impl GCalendar {
    pub async fn new(
        db: Pool<AsyncPgConnection>,
        calendar_update_tx: Sender<UpdateCalendarEvent>,
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
            calendar_update_tx,
        })
    }

    pub fn init_threads(self, worker_thread_rx: Receiver<CalendarCommands>) -> Self {
        self.new_update_calendars_thread()
            .new_worker_thread(worker_thread_rx)
    }
}

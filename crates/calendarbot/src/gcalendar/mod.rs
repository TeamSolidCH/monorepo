pub mod update_calendar_event;

use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use google_calendar3::hyper::client::HttpConnector;
use google_calendar3::{hyper, hyper_rustls, oauth2, CalendarHub, Result};

pub struct GCalendar {
    pub hub: CalendarHub<hyper_rustls::HttpsConnector<HttpConnector>>,
    pub db: Pool<ConnectionManager<PgConnection>>,
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
                    .https_only()
                    .enable_http1()
                    .build(),
            ),
            authenticator,
        );
        Ok(GCalendar { db, hub })
    }
}

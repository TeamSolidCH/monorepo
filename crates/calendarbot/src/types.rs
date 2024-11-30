/*
Calendarbot  Copyright (C) 2023 Zbinden Yohan

This program comes with ABSOLUTELY NO WARRANTY; for details type `show w'.
This is free software, and you are welcome to redistribute it
 */

use anyhow::{anyhow, Error, Result};
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::AsyncPgConnection;
use google_calendar3::{
    api::Event,
    chrono::{DateTime, Datelike, NaiveDate, NaiveTime, Utc},
};
use log::warn;
use poise::serenity_prelude as serenity;
use std::{collections::BTreeMap, env};
use tokio::sync::mpsc::Sender;

use crate::events::CalendarCommands;

// This struct holds the data that is passed to every command handler.
pub struct Data {
    pub application_id: serenity::UserId,
    pub client_id: serenity::UserId,
    pub bot_start_time: std::time::Instant,
    pub db: Pool<AsyncPgConnection>,
    pub gcalendar_tx: Sender<CalendarCommands>,
}

impl Data {
    pub fn new(
        db_connection: Pool<AsyncPgConnection>,
        gcalendar_tx: Sender<CalendarCommands>,
    ) -> Result<Data> {
        Ok(Self {
            application_id: env::var("APPLICATION_ID")
                .expect("APPLICATION_ID not found")
                .parse::<u64>()?
                .into(),
            client_id: env::var("CLIENT_ID")
                .expect("CLIENT_ID not found")
                .parse::<u64>()?
                .into(),
            bot_start_time: std::time::Instant::now(),
            db: db_connection,
            gcalendar_tx,
        })
    }
}

pub type Context<'a> = poise::Context<'a, Data, Error>;

pub const EMBED_COLOR: (u8, u8, u8) = (0xb7, 0x47, 0x00);

#[derive(Debug, Copy, Clone)]
pub enum CalendarEventSource {
    GoogleCalendar,
}

#[derive(Clone, Debug)]
pub struct CalendarEvent {
    pub id: String,
    pub summary: String,
    pub description: String,
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
    pub event_source: CalendarEventSource,
}

impl CalendarEvent {
    pub fn to_embed(events: Vec<Self>) -> serenity::CreateEmbed {
        let mut sorted: BTreeMap<(NaiveDate, NaiveDate), Vec<CalendarEvent>> = BTreeMap::new();
        let mut fields: Vec<(String, String, bool)> = vec![];

        for ele in events {
            let ele_clone = ele.clone();

            let start_date = ele_clone.start;
            let end_date = ele_clone.end;

            if let (None, None) = (start_date.as_ref(), end_date.as_ref()) {
                warn!(
                    "Event start date or event end date is None {:?}",
                    ele.clone()
                );
                continue;
            }

            let start_date = start_date.unwrap();
            let end_date = end_date.unwrap();

            sorted
                .entry((start_date.date_naive(), end_date.date_naive()))
                .or_default()
                .push(ele);
        }

        for ((start_date, end_date), events) in sorted.iter() {
            let mut field = String::new();
            for event in events {
                if event.start.is_none() {
                    warn!("Event start is None");
                    continue;
                };

                if event.end.is_none() {
                    warn!("Event end is None");
                    continue;
                };

                field.push_str(&format!(
                    "```{} - {} | {}```\n",
                    event
                        .start
                        .unwrap_or_else(|| start_date
                            .clone()
                            .and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
                            .and_utc())
                        .format("%H:%M"),
                    event
                        .end
                        .unwrap_or_else(|| end_date
                            .clone()
                            .and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
                            .and_utc())
                        .format("%H:%M"),
                    event.summary.clone()
                ));
            }
            let mut format = String::from("**%A** - %e %B");
            if start_date.year() != end_date.year() || start_date.year() != Utc::now().year() {
                format = String::from("%F");
            }

            let mut key = start_date.format(&format.clone()).to_string();

            if start_date.format("%F").to_string() != end_date.format("%F").to_string() {
                key.push_str(
                    &end_date
                        .format(format!(" // {}", format.clone()).as_str())
                        .to_string(),
                );
            }

            fields.push((key, field, false));
        }

        serenity::CreateEmbed::new().title("Events").fields(fields)
    }
}

impl PartialEq for CalendarEvent {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.summary == other.summary
            && self.description == other.description
            && self.start == other.start
            && self.end == other.end
    }
}

impl Eq for CalendarEvent {}

impl TryFrom<Event> for CalendarEvent {
    fn try_from(value: Event) -> Result<Self, Self::Error> {
        let id = match value.id {
            Some(id) => id,
            None => return Err(anyhow!("Event id is missing")),
        };

        let summary = value.summary.unwrap_or_default();
        let description = value.description.unwrap_or_default();

        let start = match value.start {
            Some(start) => start.date_time,
            None => None,
        };

        let end = match value.end {
            Some(end) => end.date_time,
            None => None,
        };

        Ok(Self {
            id,
            summary,
            description,
            start,
            end,
            event_source: CalendarEventSource::GoogleCalendar,
        })
    }

    type Error = anyhow::Error;
}

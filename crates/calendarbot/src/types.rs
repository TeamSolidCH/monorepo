/*
Calendarbot  Copyright (C) 2023 Zbinden Yohan

This program comes with ABSOLUTELY NO WARRANTY; for details type `show w'.
This is free software, and you are welcome to redistribute it
 */

use anyhow::{Error, Result};
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::AsyncPgConnection;
use google_calendar3::{
    api::Event,
    chrono::{DateTime, Utc},
};
use poise::serenity_prelude as serenity;
use std::env;
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

pub enum CalendarEventSource {
    GoogleCalendar,
}

pub struct CalendarEvent {
    pub id: String,
    pub summary: String,
    pub description: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub event_source: CalendarEventSource,
}

impl From<Event> for CalendarEvent {
    fn from(value: Event) -> Self {
        Self {
            id: value.id.unwrap(),
            summary: value.summary.unwrap_or_default(),
            description: value.description.unwrap_or_default(),
            start: value.start.as_ref().unwrap().date_time.unwrap(),
            end: value.end.as_ref().unwrap().date_time.unwrap(),
            event_source: CalendarEventSource::GoogleCalendar,
        }
    }
}

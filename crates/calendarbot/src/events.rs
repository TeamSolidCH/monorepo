/*
Calendarbot  Copyright (C) 2023 Zbinden Yohan

This program comes with ABSOLUTELY NO WARRANTY; for details type `show w'.
This is free software, and you are welcome to redistribute it
 */

use crate::types::{CalendarEvent, CalendarOptions};
use anyhow::Result;
use tokio::sync::oneshot::Sender;

pub struct VerifyCalendarEvent {
    pub calendar_id: String,
}

#[derive(Debug)]
pub enum CalendarCommands {
    VerifyCalendarId {
        calendar_id: String,
        resp: Responder<bool>,
    },
}

type Responder<T> = Sender<Result<T>>;

pub struct UpdateCalendarEvent {
    pub calendar_id: String,
    pub new_events: Vec<CalendarEvent>,
    pub calendar_options: CalendarOptions,
    pub discord_channel_and_message_ids: Vec<(u64, Option<u64>)>,
}

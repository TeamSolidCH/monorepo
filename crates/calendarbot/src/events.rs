/*
Calendarbot  Copyright (C) 2023 Zbinden Yohan

This program comes with ABSOLUTELY NO WARRANTY; for details type `show w'.
This is free software, and you are welcome to redistribute it
 */

use google_calendar3::api::Event;

pub struct VerifyCalendarEvent {
    pub calendar_id: String,
}

pub struct UpdateCalendarEvent {
    pub calendar_id: String,
    pub new_events: Vec<Event>,
    pub discord_channel_and_message_ids: Vec<(u64, Option<u64>)>,
}

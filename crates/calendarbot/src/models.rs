/*
Calendarbot  Copyright (C) 2023 Zbinden Yohan

This program comes with ABSOLUTELY NO WARRANTY; for details type `show w'.
This is free software, and you are welcome to redistribute it
 */
#![allow(non_snake_case)]

use diesel::prelude::*;

#[derive(Queryable, Selectable, Identifiable, PartialEq, Debug, Clone)]
#[diesel(table_name = crate::schema::calendars)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Calendar {
    pub id: i32,
    #[allow(non_snake_case)]
    pub googleId: String,
}

#[derive(Identifiable, Queryable, Selectable, Associations, Debug, Clone)]
#[diesel(belongs_to(Calendar))]
#[diesel(belongs_to(Guild))]
#[diesel(table_name = crate::schema::guilds_calendars)]
#[diesel(primary_key(guild_id, calendar_id, channelId))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct GuildCalendar {
    pub guild_id: i32,
    pub calendar_id: i32,
    pub channelId: String,
    pub messageId: Option<String>,
    pub forceUpdate: bool,
    pub timezone: String,
    pub pollInterval: i32,
    pub nbDisplayedDays: i32,
    pub skipWeekend: bool,
    pub skipEmptyDays: bool,
}

#[derive(Queryable, Selectable, Identifiable, PartialEq, Debug, Clone)]
#[diesel(table_name = crate::schema::guilds)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Guild {
    pub id: i32,
    pub discordId: String,
}

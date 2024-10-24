/*
Calendarbot  Copyright (C) 2023 Zbinden Yohan

This program comes with ABSOLUTELY NO WARRANTY; for details type `show w'.
This is free software, and you are welcome to redistribute it
 */

use diesel::prelude::*;

#[derive(Queryable, Selectable, Identifiable, PartialEq, Debug)]
#[diesel(table_name = crate::schema::calendars)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Calendar {
    pub id: i32,
    pub googleid: String,
    pub timezone: String,
    pub pollinterval: i32,
}

#[derive(Identifiable, Queryable, Selectable, Associations, Debug)]
#[diesel(belongs_to(Calendar))]
#[diesel(belongs_to(Guild))]
#[diesel(table_name = crate::schema::guilds_calendars)]
#[diesel(primary_key(guild_id, calendar_id, channelid))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct GuildCalendar {
    pub guild_id: i32,
    pub calendar_id: i32,
    pub channelid: String,
    pub messageid: Option<String>,
    pub forceupdate: bool,
}

#[derive(Queryable, Selectable, Identifiable, PartialEq, Debug)]
#[diesel(table_name = crate::schema::guilds)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Guild {
    pub id: i32,
    pub discordid: String,
}

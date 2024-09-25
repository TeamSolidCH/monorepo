use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::calendars)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Calendar {
    pub id: i32,
    pub googleid: String,
    pub timezone: Option<String>,
    pub pollinterval: Option<i32>,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::guilds_calendars)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct GuildCalendar {
    pub guild_id: i32,
    pub calendar_id: i32,
    pub channelid: String,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::guilds)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Guild {
    pub id: i32,
    pub discordid: String,
}

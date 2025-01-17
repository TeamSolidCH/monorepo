// @generated automatically by Diesel CLI.

diesel::table! {
    calendars (id) {
        id -> Int4,
        #[max_length = 90]
        googleId -> Varchar,
    }
}

diesel::table! {
    guilds (id) {
        id -> Int4,
        #[max_length = 64]
        discordId -> Varchar,
    }
}

diesel::table! {
    guilds_calendars (guild_id, calendar_id, channelId) {
        guild_id -> Int4,
        calendar_id -> Int4,
        #[max_length = 64]
        channelId -> Varchar,
        #[max_length = 64]
        messageId -> Nullable<Varchar>,
        forceUpdate -> Bool,
        #[max_length = 60]
        timezone -> Varchar,
        pollInterval -> Int4,
        nbDisplayedDays -> Int4,
        skipWeekend -> Bool,
        skipEmptyDays -> Bool,
    }
}

diesel::joinable!(guilds_calendars -> calendars (calendar_id));
diesel::joinable!(guilds_calendars -> guilds (guild_id));

diesel::allow_tables_to_appear_in_same_query!(calendars, guilds, guilds_calendars,);

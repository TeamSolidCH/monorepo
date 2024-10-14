use google_calendar3::api::Event;

pub struct VerifyCalendarEvent {
    pub calendar_id: String,
}

pub struct UpdateCalendarEvent {
    pub calendar_id: String,
    pub new_events: Vec<Event>,
    pub discord_channel_and_message_ids: Vec<(u64, Option<u64>)>,
}

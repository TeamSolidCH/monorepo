use crate::GCalendar;

use crate::models::{Calendar, GuildCalendar};
use diesel::prelude::*;
use google_calendar3::{api::Event, chrono};
use log::{error, warn};
use tokio::sync::mpsc::Sender;

pub struct UpdateCalendarEvent {
    pub calendar_id: String,
    pub new_events: Vec<Event>,
    pub discord_channel_and_message_ids: Vec<(u64, Option<u64>)>,
}

impl GCalendar {
    pub async fn update_calendars(&self, sender: Sender<UpdateCalendarEvent>) {
        use crate::schema::calendars::dsl::*;

        let db = &mut self.db.clone().get();

        if let Err(e) = db {
            warn!("Unable to clone db: {:?}", e);
            return;
        }

        let db = db.as_mut().unwrap();

        let db_calendars = calendars
            .select(Calendar::as_select())
            .load(db)
            .expect("Unable to get calendars");

        for calendar in db_calendars {
            let cal_id = calendar.googleid.clone();
            let sender = sender.clone();
            let events = self
                .hub
                .events()
                .list(&cal_id)
                .time_min(chrono::Utc::now())
                .doit()
                .await
                .expect("Unable to get events")
                .1;

            let guild_calendars = GuildCalendar::belonging_to(&calendar)
                .select(GuildCalendar::as_select())
                .load(db)
                .expect("Unable to get channel and message ids");

            let mut discord_channel_and_message_ids = Vec::new();

            for guild_calendar in guild_calendars {
                let channel_id = guild_calendar.channelid.parse::<u64>();
                if let Err(e) = channel_id {
                    error!("Unable to parse channel id: {:?}", e);
                    continue;
                }
                let msg_id = if let Some(val) = guild_calendar.messageid {
                    match val.parse::<u64>() {
                        Err(e) => {
                            warn!("Unable to parse message id: {:?}", e);
                            None
                        }
                        Ok(parsed_val) => Some(parsed_val),
                    }
                } else {
                    None
                };

                discord_channel_and_message_ids.push((channel_id.unwrap(), msg_id))
            }

            sender
                .send(UpdateCalendarEvent {
                    discord_channel_and_message_ids,
                    calendar_id: cal_id,
                    new_events: events.items.unwrap_or_default(),
                })
                .await
                .expect("Unable to send events");
        }
    }

    pub fn compare_event(a: &Event, b: &Event) -> bool {
        a.id == b.id
            && a.summary == b.summary
            && a.description == b.description
            && a.location == b.location
            && a.start.as_ref().unwrap().date_time.as_ref().unwrap()
                == b.start.as_ref().unwrap().date_time.as_ref().unwrap()
            && a.end.as_ref().unwrap().date_time.as_ref().unwrap()
                == b.end.as_ref().unwrap().date_time.as_ref().unwrap()
    }
}

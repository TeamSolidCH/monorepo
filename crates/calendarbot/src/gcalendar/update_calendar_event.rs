/*
Calendarbot  Copyright (C) 2023 Zbinden Yohan

This program comes with ABSOLUTELY NO WARRANTY; for details type `show w'.
This is free software, and you are welcome to redistribute it
 */

use crate::GCalendar;

use crate::events::UpdateCalendarEvent;
use crate::models::{Calendar, GuildCalendar};
use crate::schema::guilds_calendars;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use google_calendar3::{api::Event, chrono};
use log::{debug, error, trace, warn};
use std::time::Duration;

impl GCalendar {
    pub(crate) fn new_update_calendars_thread(self) -> Self {
        let mut self_clone = self.clone();
        tokio::spawn(async move {
            loop {
                self_clone.update_calendars().await;
                trace!("Updated calendars");
                tokio::time::sleep(Duration::from_secs(10)).await;
            }
        });
        self
    }

    async fn update_calendars(&mut self) {
        let db = &mut self.db.clone().get().await;
        if let Err(e) = db {
            warn!("Unable to clone db: {:?}", e);
            return;
        }

        let db = db.as_mut().unwrap();
        use crate::schema::calendars::dsl::*;

        let db_calendars = calendars
            .select(Calendar::as_select())
            .load(db)
            .await
            .expect("Unable to get calendars");

        for calendar in db_calendars {
            trace!("Updating calendar: {}", calendar.googleid);
            let cal_id = calendar.googleid.clone();
            let sender = self.calendar_update_tx.clone();
            let events = self
                .hub
                .clone()
                .events()
                .list(&cal_id)
                .time_min(chrono::Utc::now())
                .doit()
                .await
                .expect("Unable to get events")
                .1;

            let new_events = events.items.clone().unwrap_or_default();

            let cached_events = self.events_cache.entry(cal_id.clone()).or_default();
            let matching = cached_events
                .iter()
                .zip(new_events.iter())
                .filter(|&(a, b)| GCalendar::compare_event(a, b))
                .count();

            let do_match = matching == new_events.len() && matching == cached_events.len();
            trace!(
                "matching: {} == {} && {} == {}",
                matching,
                new_events.len(),
                matching,
                cached_events.len()
            );

            let mut guild_calendars = GuildCalendar::belonging_to(&calendar)
                .select(GuildCalendar::as_select())
                .load(db)
                .await
                .expect("Unable to get channel and message ids");

            let forced_update = guild_calendars
                .iter()
                .any(|guild_calendar| guild_calendar.forceupdate);

            if do_match && !cached_events.is_empty() && !forced_update {
                debug!("No new events");
                continue;
            }

            // if it is a forced update only update these channels
            if forced_update {
                guild_calendars = guild_calendars
                    .into_iter()
                    .filter(|guild_calendar| guild_calendar.forceupdate)
                    .collect::<Vec<GuildCalendar>>();
            }

            // Add new events to cache
            if !cached_events.is_empty() {
                cached_events.clear();
            }
            cached_events.extend(new_events.clone());

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
                    new_events,
                })
                .await
                .expect("Unable to send events");

            // If was forced update change to false in db
            if forced_update {
                let res = diesel::update(GuildCalendar::belonging_to(&calendar))
                    .set(guilds_calendars::forceupdate.eq(false))
                    .execute(db)
                    .await;

                if let Err(e) = res {
                    error!("Unable to change forceUpdate to false: {}", e);
                }
            }
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

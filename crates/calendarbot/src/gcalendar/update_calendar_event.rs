/*
Calendarbot  Copyright (C) 2023 Zbinden Yohan

This program comes with ABSOLUTELY NO WARRANTY; for details type `show w'.
This is free software, and you are welcome to redistribute it
 */

use crate::types::{CalendarEvent, CalendarOptions};
use crate::GCalendar;
use anyhow::Error;

use crate::events::UpdateCalendarEvent;
use crate::models::{Calendar, GuildCalendar};
use crate::schema::guilds_calendars;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use google_calendar3::chrono;
use log::{debug, error, trace, warn};
use std::collections::BTreeMap;
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
            trace!("Updating calendar: {}", calendar.googleId);
            let cal_id = calendar.googleId.clone();
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

            let new_events: Vec<CalendarEvent> = events
                .items
                .unwrap_or_default()
                .into_iter()
                .map(|event| {
                    CalendarEvent::try_from(event)
                        .map_err(Error::msg)
                        .expect("Unable to convert event")
                })
                .collect();

            let cached_events = self.events_cache.entry(cal_id.clone()).or_default();
            let matching = cached_events
                .iter()
                .zip(new_events.iter())
                .filter(|&(a, b)| a == b)
                .count();

            let do_match = matching == new_events.len() && matching == cached_events.len();
            trace!(
                "matching: ({} == {} && {} == {}) = {}",
                matching,
                new_events.len(),
                matching,
                cached_events.len(),
                do_match
            );

            let mut guild_calendars = GuildCalendar::belonging_to(&calendar)
                .select(GuildCalendar::as_select())
                .load(db)
                .await
                .expect("Unable to get channel and message ids");

            let forced_update = guild_calendars
                .iter()
                .any(|guild_calendar| guild_calendar.forceUpdate);

            if do_match && !forced_update {
                debug!("No new events");
                continue;
            }

            // if it is a forced update only update these channels
            if forced_update {
                guild_calendars = guild_calendars
                    .into_iter()
                    .filter(|guild_calendar| guild_calendar.forceUpdate)
                    .collect::<Vec<GuildCalendar>>();
            }

            // Add new events to cache
            if !cached_events.is_empty() {
                cached_events.clear();
            }
            cached_events.extend(new_events.clone());

            let mut discord_channel_and_message_ids = BTreeMap::new();

            for guild_calendar in guild_calendars {
                let channel_id = guild_calendar.channelId.parse::<u64>();
                if let Err(e) = channel_id {
                    error!("Unable to parse channel id: {:?}", e);
                    continue;
                }
                let msg_id = if let Some(ref val) = guild_calendar.messageId {
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

                let options = CalendarOptions::try_from(guild_calendar.clone());
                if options.is_err() {
                    error!("Unable to convert CalendarOptions: {:?}", options.err());
                    continue;
                }

                discord_channel_and_message_ids
                    .entry(options.unwrap())
                    .and_modify(|e: &mut Vec<(u64, Option<u64>)>| {
                        e.push((channel_id.clone().unwrap(), msg_id))
                    })
                    .or_insert_with(|| vec![(channel_id.clone().unwrap(), msg_id)]);
            }

            for (options, discord_channel_and_message_ids) in discord_channel_and_message_ids {
                sender
                    .send(UpdateCalendarEvent {
                        discord_channel_and_message_ids,
                        calendar_id: cal_id.clone(),
                        calendar_options: options,
                        new_events: new_events.clone(),
                    })
                    .await
                    .expect("Unable to send events");
            }

            // If was forced update change to false in db
            if forced_update {
                let res = diesel::update(GuildCalendar::belonging_to(&calendar))
                    .set(guilds_calendars::forceUpdate.eq(false))
                    .execute(db)
                    .await;

                if let Err(e) = res {
                    error!("Unable to change forceUpdate to false: {}", e);
                }
            }
        }
    }
}

use crate::discord::{Discord, LocalCache};
use crate::UpdateCalendarEvent;

use crate::schema::calendars::dsl as calendars;
use crate::schema::guilds_calendars::dsl as guilds_calendars;
use anyhow::Result;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use google_calendar3::api::Event;
use google_calendar3::chrono::{Datelike, NaiveDate, NaiveTime, Utc};
use log::{debug, error, warn};
use poise::serenity_prelude as serenity;
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

impl Discord {
    /// Send or edit a message in a channel
    async fn send_or_edit_message(
        channel_id: u64,
        message_id: Option<u64>,
        embed: serenity::CreateEmbed,
        cache: LocalCache,
    ) -> Result<serenity::MessageId> {
        let channel = serenity::ChannelId::new(channel_id);
        if let Some(message_id) = message_id {
            let msg_id = serenity::MessageId::new(message_id);
            debug!("Trying to edit message ({})", msg_id.get());
            let result = cache
                .client
                .edit_message(
                    channel,
                    msg_id,
                    &serenity::EditMessage::new().add_embed(embed.clone()),
                    Vec::new(),
                )
                .await;

            if let Err(e) = result {
                error!("Failed to edit message ({}): {}", message_id.clone(), e);
            } else {
                return Ok(msg_id);
            }
        }

        debug!("Send new message");
        match channel
            .send_message(cache, serenity::CreateMessage::new().add_embed(embed))
            .await
        {
            Err(e) => Err(e.into()),
            Ok(res) => Ok(res.id),
        }
    }

    pub(crate) fn calendar_events_thread(
        mut calendar_rx: mpsc::Receiver<UpdateCalendarEvent>,
        cache: Arc<Mutex<Option<LocalCache>>>,
        db: Pool<ConnectionManager<PgConnection>>,
    ) {
        //TODO: Find a better way to handle new events (maybe a threadpool)
        tokio::spawn(async move {
            let db = db.get();
            if let Err(e) = db {
                error!("Unable to get db connection from poolmanager: {:?}", e);
                return;
            }

            let mut db = db.unwrap();

            while let Some(event) = calendar_rx.recv().await {
                debug!("Received event for calendar {}", event.calendar_id);

                let cache = cache.as_ref().lock().await.clone().unwrap();

                let embed = match Discord::event_to_embed(event.new_events.clone()) {
                    Ok(v) => v,
                    Err(_) => serenity::CreateEmbed::new().title("Events").field(
                        "Error",
                        "Error getting events",
                        true,
                    ),
                };

                for (channel_id, message_id) in event.discord_channel_and_message_ids {
                    debug!(target: &channel_id.to_string(), "Handling new_events with message_id: {:?}", message_id);
                    let result = Discord::send_or_edit_message(
                        channel_id,
                        message_id,
                        embed.clone(),
                        cache.clone(),
                    )
                    .await;

                    match result {
                        Err(e) => {
                            error!("Failed to send or edit message: {}, calendar: {}, channel_id: {:?}, message_id: {:?}", e, event.calendar_id, channel_id, message_id);
                        }
                        Ok(msg_id) => {
                            // We need to update the message_id in the database
                            if Some(msg_id.get()) != message_id {
                                let db_cal_id = calendars::calendars
                                    .filter(calendars::googleid.eq(event.calendar_id.clone()))
                                    .select(calendars::id)
                                    .first::<i32>(&mut db)
                                    .expect("Unable to get calendar id from database");

                                diesel::update(guilds_calendars::guilds_calendars)
                                    .filter(
                                        guilds_calendars::channelid
                                            .eq(channel_id.to_string())
                                            .and(guilds_calendars::calendar_id.eq(db_cal_id)),
                                    )
                                    .set(guilds_calendars::messageid.eq(msg_id.get().to_string()))
                                    .execute(&mut db)
                                    .expect("Unable to update message id in database");
                            }
                        }
                    };
                }
            }
        });
    }

    fn event_to_embed(events: Vec<Event>) -> Result<serenity::CreateEmbed> {
        let mut sorted: BTreeMap<(NaiveDate, NaiveDate), Vec<Event>> = BTreeMap::new();
        let mut fields: Vec<(String, String, bool)> = vec![];

        for ele in events {
            let ele_clone = ele.clone();

            let start_date = ele_clone.start;
            let end_date = ele_clone.end;

            if let (None, None) = (start_date.as_ref(), end_date.as_ref()) {
                warn!(
                    "Event start date or event end date is None {:?}",
                    ele.clone()
                );
                continue;
            }

            let start_date = start_date.unwrap().date_time;
            let end_date = end_date.unwrap().date_time;

            if let (None, None) = (start_date, end_date) {
                warn!(
                    "Event start datetime or event end datetime is None {:?}",
                    ele.clone()
                );
                continue;
            }

            sorted
                .entry((
                    start_date.unwrap().date_naive(),
                    end_date.unwrap().date_naive(),
                ))
                .or_default()
                .push(ele);
        }

        for ((start_date, end_date), events) in sorted.iter() {
            let mut field = String::new();
            for event in events {
                match event.start {
                    None => {
                        warn!("Event start is None");
                        continue;
                    }
                    _ => (),
                }

                match event.end {
                    None => {
                        warn!("Event end is None");
                        continue;
                    }
                    _ => (),
                }

                field.push_str(&format!(
                    "```{} - {} | {}```\n",
                    event
                        .start
                        .clone()
                        .unwrap()
                        .date_time
                        .unwrap_or_else(|| start_date
                            .clone()
                            .and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
                            .and_utc())
                        .format("%H:%M"),
                    event
                        .end
                        .clone()
                        .unwrap()
                        .date_time
                        .unwrap_or_else(|| end_date
                            .clone()
                            .and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
                            .and_utc())
                        .format("%H:%M"),
                    event.summary.clone().unwrap()
                ));
            }
            let mut format = String::from("**%A** - %e %B");
            if start_date.year() != end_date.year() || start_date.year() != Utc::now().year() {
                format = String::from("%F");
            }

            let mut key = start_date.format(&format.clone()).to_string();

            if start_date.format("%F").to_string() != end_date.format("%F").to_string() {
                key.push_str(
                    &end_date
                        .format(format!(" // {}", format.clone()).as_str())
                        .to_string(),
                );
            }

            fields.push((key, field, false));
        }

        Ok(serenity::CreateEmbed::new().title("Events").fields(fields))
    }
}

use crate::discord::{Discord, LocalCache};
use crate::gcalendar::GCalendar;
use crate::UpdateCalendarEvent;

use anyhow::Result;
use google_calendar3::api::Event;
use google_calendar3::chrono::{Datelike, NaiveDate, NaiveTime, Utc};
use log::{debug, error, trace, warn};
use poise::serenity_prelude as serenity;
use std::collections::{btree_map::Entry, BTreeMap};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

impl Discord {
    pub(crate) fn calendar_events_thread(
        mut calendar_rx: mpsc::Receiver<UpdateCalendarEvent>,
        cache: Arc<Mutex<Option<LocalCache>>>,
    ) {
        //TODO: Find a better way to handle new events (maybe a threadpool)
        tokio::spawn(async move {
            let mut events_cache: BTreeMap<String, Vec<Event>> = BTreeMap::new();

            // TODO: The message cache should be persisted between restarts (maybe redis or store
            // in db ?)
            let mut message_cache: BTreeMap<String, u64> = BTreeMap::new();

            let channel = serenity::ChannelId::new(1102198299093647470);

            while let Some(event) = calendar_rx.recv().await {
                debug!("Received event for calendar {}", event.calendar_id);

                let events = events_cache.entry(event.calendar_id.clone()).or_default();
                let matching = events
                    .iter()
                    .zip(event.new_events.iter())
                    .filter(|&(a, b)| GCalendar::compare_event(a, b))
                    .count();

                let do_match = matching == event.new_events.len() && matching == events.len();
                trace!(
                    "matching: {} == {} && {} == {}",
                    matching,
                    event.new_events.len(),
                    matching,
                    events.len()
                );

                if do_match && !events.is_empty() {
                    debug!("No new events");
                    continue;
                }

                let cache = cache.as_ref().lock().await.clone().unwrap();
                let embed = match Discord::event_to_embed(event.new_events.clone()) {
                    Ok(v) => v,
                    Err(_) => serenity::CreateEmbed::new().title("Events").field(
                        "Error",
                        "Error getting events",
                        true,
                    ),
                };

                let mut create_new_message = false;
                if let Entry::Occupied(o) = message_cache.entry(event.calendar_id.clone()) {
                    let message_id = serenity::MessageId::new(*o.get());

                    let err = cache
                        .client
                        .edit_message(
                            channel,
                            message_id,
                            &serenity::EditMessage::new().add_embed(embed.clone()),
                            Vec::new(),
                        )
                        .await;

                    if let Err(e) = err {
                        error!("Failed to edit message ({}): {}", message_id.clone(), e);
                        create_new_message = true;
                    };
                } else {
                    create_new_message = true;
                }

                if create_new_message {
                    let res = channel
                        .send_message(cache, serenity::CreateMessage::new().add_embed(embed))
                        .await
                        .unwrap();

                    message_cache
                        .entry(event.calendar_id.clone())
                        .or_insert(res.id.get());
                }

                if !events.is_empty() {
                    events.clear();
                }
                events.extend(event.new_events.clone());
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
